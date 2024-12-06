"""Usage:
# Save directly to KRec file:
python examples/test_krec_save.py --output_krec output_save_test.krec -v

# Combine with video:
python examples/test_krec_save.py \
    --output_krec output_save_test.krec \
    --input_video /path/to/video.mkv \
    --output_video output_combined.mkv \
    -v
"""

import krec
import argparse
import numpy as np
from pathlib import Path
import uuid
import time
import logging


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


def verify_krec_data(original_data, loaded_krec):
    """Verify that the loaded KRec matches the original data"""
    logging.info("Verifying loaded KRec data...")

    timestamps = original_data["timestamps"]
    num_frames = len(loaded_krec)

    for i in range(num_frames):
        frame = loaded_krec[i]

        # Verify actuator states
        for j, state in enumerate(frame.get_actuator_states()):
            expected_pos = original_data["position_waves"][j][i]
            expected_vel = original_data["velocity_waves"][j][i]
            expected_torque = original_data["torque_waves"][j][i]

            if not np.allclose([state.position], [expected_pos], rtol=1e-5):
                logging.error(f"Position mismatch at frame {i}, joint {j}: {state.position} != {expected_pos}")
                return False

            if not np.allclose([state.velocity], [expected_vel], rtol=1e-5):
                logging.error(f"Velocity mismatch at frame {i}, joint {j}: {state.velocity} != {expected_vel}")
                return False

            if not np.allclose([state.torque], [expected_torque], rtol=1e-5):
                logging.error(f"Torque mismatch at frame {i}, joint {j}: {state.torque} != {expected_torque}")
                return False

        # Verify IMU data
        imu = frame.get_imu_values()
        if imu:
            expected_accel = [
                original_data["accel_waves"]["x"][i],
                original_data["accel_waves"]["y"][i],
                original_data["accel_waves"]["z"][i],
            ]
            expected_gyro = [
                original_data["gyro_waves"]["x"][i],
                original_data["gyro_waves"]["y"][i],
                original_data["gyro_waves"]["z"][i],
            ]

            actual_accel = [imu.accel.x, imu.accel.y, imu.accel.z]
            actual_gyro = [imu.gyro.x, imu.gyro.y, imu.gyro.z]

            if not np.allclose(actual_accel, expected_accel, rtol=1e-5):
                logging.error(f"Acceleration mismatch at frame {i}: {actual_accel} != {expected_accel}")
                return False

            if not np.allclose(actual_gyro, expected_gyro, rtol=1e-5):
                logging.error(f"Gyro mismatch at frame {i}: {actual_gyro} != {expected_gyro}")
                return False

    logging.info("All data verified successfully!")
    return True


def main(args):
    logging.info(f"Creating synthetic KRec with sine waves...")
    synthetic_krec, original_data = create_sine_wave_krec()

    # Save KRec file
    logging.info(f"Saving KRec to: {args.output_krec}")
    synthetic_krec.save(args.output_krec)

    # If video paths are provided, combine with video
    if args.input_video and args.output_video:
        logging.info(f"Combining video from {args.input_video} with KRec...")
        try:
            krec.combine_with_video(
                args.input_video,
                args.output_krec,
                args.output_video,
            )
            logging.info(f"Successfully combined video and KRec to: {args.output_video}")

            # Verify the output by trying to extract the KRec
            logging.info("Verifying output by extracting KRec...")
            extracted_krec = krec.extract_from_video(args.output_video, verbose=args.verbose)
            logging.info(f"Successfully extracted KRec with {len(extracted_krec)} frames")

            # Verify the data matches
            if not verify_krec_data(original_data, extracted_krec):
                logging.error("Data verification failed!")
                return 1

        except Exception as e:
            logging.error(f"Error during video processing: {e}")
            return 1
    else:
        # Load and verify the direct KRec save
        logging.info(f"Loading KRec from: {args.output_krec}")
        loaded_krec = krec.KRec.load(args.output_krec)
        logging.info(f"Loaded KRec with {len(loaded_krec)} frames")

        # Verify the data matches
        if not verify_krec_data(original_data, loaded_krec):
            logging.error("Data verification failed!")
            return 1

    return 0


if __name__ == "__main__":
    # Configure logging
    logging.basicConfig(level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s")

    parser = argparse.ArgumentParser(description="Save and verify KRec data")
    parser.add_argument("--output_krec", type=str, required=True, help="Output KRec file path")
    parser.add_argument("--input_video", type=str, help="Input video file path (optional)")
    parser.add_argument("--output_video", type=str, help="Output video file path (optional)")
    parser.add_argument("-v", "--verbose", action="store_true", help="Enable verbose output")

    args = parser.parse_args()

    # Validate args
    if bool(args.input_video) != bool(args.output_video):
        parser.error("--input_video and --output_video must be provided together")

    exit(main(args))
