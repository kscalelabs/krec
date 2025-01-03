"""Functions for generating synthetic KRec data."""

import logging
import time
import uuid

import krec
import numpy as np


def generate_synthetic_krec(num_frames=100, fps=30, frequency=10.0, num_actuators=10):
    """Generate a synthetic KRec object with simple wave data for 10 actuators.
    States follow commands with a one-frame delay.

    Args:
        num_frames: Number of frames to generate
        fps: Frames per second
        frequency: Frequency multiplier for the sine waves (higher = faster oscillation)
        num_actuators: Number of actuators to simulate (default: 10)
    """
    timestamps = np.arange(num_frames) / fps

    # Generate command waves for each actuator
    # Each actuator gets the same frequency (which decreases over time) but different phase offset
    actuator_wave = {}
    for i in range(num_actuators):
        time_varying_frequency = frequency / (1 + timestamps / 3)
        phase = time_varying_frequency * timestamps
        phase_with_offset = phase + i
        actuator_wave[i] = np.sin(phase_with_offset)

    start_time = int(time.time_ns())
    header = krec.KRecHeader(
        uuid=str(uuid.uuid4()),
        task="simple_wave_test",
        robot_platform="test_platform",
        robot_serial="test_serial_001",
        start_timestamp=start_time,
        end_timestamp=start_time + int(num_frames * (1 / fps) * 1e9),
    )

    for i in range(num_actuators):
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

    for i in range(num_frames):
        frame = krec.KRecFrame(
            video_timestamp=start_time + int(timestamps[i] * 1e9),
            video_frame_number=i,
            inference_step=i,
        )

        for j in range(num_actuators):
            command_val = actuator_wave[j][i]
            state_val = actuator_wave[j][i - 1] if i > 0 else 0.0

            command = krec.ActuatorCommand(
                actuator_id=j,
                position=command_val,
                velocity=0.0,
                torque=0.0,
            )
            frame.add_actuator_command(command)

            state = krec.ActuatorState(
                actuator_id=j,
                online=True,
                position=state_val,
                velocity=0.0,
                torque=0.0,
                temperature=25.0,
                voltage=12.0,
                current=1.0,
            )
            frame.add_actuator_state(state)

        imu_values = krec.IMUValues(
            accel=krec.Vec3(x=0.0, y=0.0, z=1.0),
            gyro=krec.Vec3(x=0.0, y=0.0, z=0.0),
            quaternion=krec.IMUQuaternion(x=0.0, y=0.0, z=0.0, w=1.0),
        )
        frame.set_imu_values(imu_values)

        krec_obj.add_frame(frame)

    return krec_obj


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s")

    logging.info("Generating synthetic KRec data...")
    krec_obj = generate_synthetic_krec()

    logging.info("Generated KRec with %d frames" % len(krec_obj))
    logging.info("UUID: %s" % krec_obj.header.uuid)
    logging.info("Task: %s" % krec_obj.header.task)
    logging.info("Robot Platform: %s" % krec_obj.header.robot_platform)
    logging.info("Start Timestamp: %d" % krec_obj.header.start_timestamp)
