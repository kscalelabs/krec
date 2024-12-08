"""Usage:
# Visualize synthetic KRec data:
python examples/krec_to_rrd.py --synthetic --spawn-viewer -v

# Save to RRD file:
python examples/krec_to_rrd.py --synthetic --output output.rrd -v
"""

import argparse
import logging
import time
from pathlib import Path
import numpy as np
import rerun as rr
import krec
import uuid

def create_sine_wave_krec(num_frames=100, fps=30):
    # Create timestamps
    timestamps = np.arange(num_frames) / fps

    # Create sine waves
    position_waves = {
        0: np.sin(2 * np.pi * 0.5 * timestamps),
        1: np.sin(2 * np.pi * 0.5 * timestamps),
        2: np.sin(2 * np.pi * 0.5 * timestamps),
    }
    velocity_waves = {
        0: np.sin(2 * np.pi * 0.5 * timestamps),
        1: np.sin(2 * np.pi * 0.5 * timestamps),
        2: np.sin(2 * np.pi * 0.5 * timestamps),
    }
    torque_waves = {
        0: 0.5 * np.sin(2 * np.pi * 0.5 * timestamps),
        1: 0.5 * np.sin(2 * np.pi * 0.5 * timestamps),
        2: 0.5 * np.sin(2 * np.pi * 0.5 * timestamps),
    }

    accel_waves = {
        "x": 0.1 * np.sin(2 * np.pi * 0.5 * timestamps),
        "y": 0.1 * np.sin(2 * np.pi * 0.5 * timestamps),
        "z": 9.81 + 0.1 * np.sin(2 * np.pi * 0.5 * timestamps),
    }
    gyro_waves = {
        "x": np.sin(2 * np.pi * 0.5 * timestamps),
        "y": np.sin(2 * np.pi * 0.5 * timestamps),
        "z": np.sin(2 * np.pi * 0.5 * timestamps),
    }

    # Create KRec with header
    start_time = int(time.time_ns())
    header = krec.KRecHeader(
        uuid=str(uuid.uuid4()),
        task="sine_wave_test",
        robot_platform="test_platform",
        robot_serial="test_serial_001",
        start_timestamp=start_time,
        end_timestamp=start_time + int(num_frames * (1 / fps) * 1e9),
    )

    # Add actuator configs
    for i in range(3):
        actuator_config = krec.ActuatorConfig(
            actuator_id=i,
            kp=1.0,
            kd=0.1,
            ki=0.01,
            max_torque=10.0,
            name=f"Joint{i}",
        )
        header.add_actuator_config(actuator_config)

    krec_obj = krec.KRec(header)

    # Add frames with sine wave data
    for i in range(num_frames):
        frame = krec.KRecFrame(
            video_timestamp=start_time + int(timestamps[i] * 1e9),
            video_frame_number=i,
            inference_step=i,
        )

        # Add actuator states and commands
        for j in range(3):
            state = krec.ActuatorState(
                actuator_id=j,
                online=True,
                position=position_waves[j][i],
                velocity=velocity_waves[j][i],
                torque=torque_waves[j][i],
                temperature=25.0 + np.sin(2 * np.pi * 0.1 * timestamps[i]),  # Slowly varying temperature
                voltage=12.0 + np.sin(2 * np.pi * 0.05 * timestamps[i]),  # Slowly varying voltage
                current=1.0 + 0.1 * np.sin(2 * np.pi * 0.2 * timestamps[i]),  # Slowly varying current
            )
            frame.add_actuator_state(state)

            command = krec.ActuatorCommand(
                actuator_id=j,
                position=position_waves[j][i],
                velocity=velocity_waves[j][i],
                torque=torque_waves[j][i],
            )
            frame.add_actuator_command(command)

        # Add IMU values
        accel = krec.Vec3(x=accel_waves["x"][i], y=accel_waves["y"][i], z=accel_waves["z"][i])
        gyro = krec.Vec3(x=gyro_waves["x"][i], y=gyro_waves["y"][i], z=gyro_waves["z"][i])
        # Simple rotation quaternion (could be made more complex if needed)
        quaternion = krec.IMUQuaternion(x=0.0, y=0.0, z=np.sin(timestamps[i] / 2), w=np.cos(timestamps[i] / 2))
        imu_values = krec.IMUValues(accel=accel, gyro=gyro, quaternion=quaternion)
        frame.set_imu_values(imu_values)

        krec_obj.add_frame(frame)

    return krec_obj, {
        "position_waves": position_waves,
        "velocity_waves": velocity_waves,
        "torque_waves": torque_waves,
        "accel_waves": accel_waves,
        "gyro_waves": gyro_waves,
        "timestamps": timestamps,
    }


def log_frame_data(frame, frame_idx):
    """Log frame data to Rerun visualization."""
    
    # Set time sequence
    rr.set_time_sequence("frame_idx", frame_idx)
    
    # Log metadata
    rr.log("metadata/video_frame_number", rr.Scalar(frame.video_frame_number))
    rr.log("metadata/timestamp", rr.Scalar(frame.video_timestamp))
    rr.log("metadata/inference_step", rr.Scalar(frame.inference_step))

    # Log actuator states
    for state in frame.get_actuator_states():
        prefix = f"actuators/state_{state.actuator_id}"
        rr.log(f"{prefix}/position", rr.Scalar(state.position))
        rr.log(f"{prefix}/velocity", rr.Scalar(state.velocity))
        rr.log(f"{prefix}/torque", rr.Scalar(state.torque))
        rr.log(f"{prefix}/temperature", rr.Scalar(state.temperature))
        rr.log(f"{prefix}/voltage", rr.Scalar(state.voltage))
        rr.log(f"{prefix}/current", rr.Scalar(state.current))
        rr.log(f"{prefix}/online", rr.Scalar(float(state.online)))

    # Log actuator commands
    for cmd in frame.get_actuator_commands():
        prefix = f"actuators/command_{cmd.actuator_id}"
        rr.log(f"{prefix}/position", rr.Scalar(cmd.position))
        rr.log(f"{prefix}/velocity", rr.Scalar(cmd.velocity))
        rr.log(f"{prefix}/torque", rr.Scalar(cmd.torque))

    # Log IMU values if present
    imu_values = frame.get_imu_values()
    if imu_values:
        if imu_values.accel:
            # Only scalar logging for acceleration
            rr.log("imu/acceleration/x", rr.Scalar(imu_values.accel.x))
            rr.log("imu/acceleration/y", rr.Scalar(imu_values.accel.y))
            rr.log("imu/acceleration/z", rr.Scalar(imu_values.accel.z))
        
        if imu_values.gyro:

            rr.log("imu/angular_velocity/x", rr.Scalar(imu_values.gyro.x))
            rr.log("imu/angular_velocity/y", rr.Scalar(imu_values.gyro.y))
            rr.log("imu/angular_velocity/z", rr.Scalar(imu_values.gyro.z))
            
        if imu_values.quaternion:
            rr.log("imu/orientation/x", rr.Scalar(imu_values.quaternion.x))
            rr.log("imu/orientation/y", rr.Scalar(imu_values.quaternion.y))
            rr.log("imu/orientation/z", rr.Scalar(imu_values.quaternion.z))
            rr.log("imu/orientation/w", rr.Scalar(imu_values.quaternion.w))

def main(args):
    logging.info("Starting visualization process")

    try:
        # Initialize Rerun
        spawn_viewer = args.spawn_viewer and not args.output
        rr.init("krec_visualization", spawn=spawn_viewer)
        
        if args.synthetic:
            # Use synthetic data
            # extracted_krec = load_krec("path to krec.krec")
            extracted_krec, _ = create_sine_wave_krec()
            logging.info("Created synthetic KRec data")
        else:
            # Extract from file
            extracted_krec = krec.extract_from_video(args.input, verbose=args.verbose)
            logging.info("Extraction from file successful")

        logging.info(f"Processing {len(extracted_krec)} frames")

        # Log header information
        header = extracted_krec.header
        rr.log("metadata/header", 
               rr.TextDocument(
                   f"UUID: {header.uuid}\n"
                   f"Task: {header.task}\n"
                   f"Robot Platform: {header.robot_platform}\n"
                   f"Robot Serial: {header.robot_serial}\n"
                   f"Start Timestamp: {header.start_timestamp}\n"
                   f"End Timestamp: {header.end_timestamp}",
                   media_type=rr.MediaType.MARKDOWN
               ))

        # Log each frame
        for idx, frame in enumerate(extracted_krec):
            log_frame_data(frame, idx)

        # Save to RRD file if output path provided
        if args.output:
            output_path = Path(args.output)
            rr.save(str(output_path))
            logging.info(f"Saved RRD to: {output_path}")
        elif args.spawn_viewer:
            # Keep the viewer running
            try:
                while True:
                    time.sleep(1)
            except KeyboardInterrupt:
                logging.info("Ctrl-C received. Exiting.")

    except Exception as e:
        logging.error(f"Error: {e}")
        raise

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Visualize KRec data using Rerun")
    parser.add_argument("--input", type=str, help="Input KREC or KREC.MKV file path")
    parser.add_argument("-o", "--output", type=str, help="Output RRD file path (optional)")
    parser.add_argument("-v", "--verbose", action="store_true", help="Enable verbose output")
    parser.add_argument("--synthetic", action="store_true", help="Use synthetic data instead of input file")
    parser.add_argument("--spawn-viewer", action="store_true", help="Spawn Rerun viewer (ignored if --output is set)")

    logging.basicConfig(level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s")

    args = parser.parse_args()
    
    # Validate args
    if not args.synthetic and not args.input:
        parser.error("Either --synthetic or --input must be provided")

    main(args)