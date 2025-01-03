"""Tests KRec serialization in Python."""

import logging
import math
import os
import time
import uuid
from pathlib import Path
from typing import Mapping

import pytest

import krec


@pytest.fixture
def synthetic_krec_data() -> tuple[krec.KRec, dict[str, Mapping[int | str, list[float]]]]:
    """Fixture that provides a synthetic KRec object and its original data."""
    krec_obj, original_data = create_sine_wave_krec(num_frames=50, fps=30)
    return krec_obj, original_data


def create_sine_wave_krec(
    num_frames: int = 50,
    fps: int = 30,
) -> tuple[krec.KRec, dict[str, Mapping[int | str, list[float]]]]:
    """Create a synthetic KRec object with sine wave data for testing."""
    # Create timestamps
    timestamps = [i / fps for i in range(num_frames)]

    # Create wave data for each joint
    position_waves: dict[int | str, list[float]] = {
        i: [math.sin(2 * math.pi * 0.5 * t) for t in timestamps] for i in range(3)
    }
    velocity_waves: dict[int | str, list[float]] = {
        i: [math.sin(2 * math.pi * 0.5 * t) for t in timestamps] for i in range(3)
    }
    torque_waves: dict[int | str, list[float]] = {
        i: [0.5 * math.sin(2 * math.pi * 0.5 * t) for t in timestamps] for i in range(3)
    }

    # Add IMU data
    accel_waves: dict[int | str, list[float]] = {
        "x": [0.1 * math.sin(2 * math.pi * 0.5 * t) for t in timestamps],
        "y": [0.1 * math.sin(2 * math.pi * 0.5 * t) for t in timestamps],
        "z": [9.81 + 0.1 * math.sin(2 * math.pi * 0.5 * t) for t in timestamps],
    }
    gyro_waves: dict[int | str, list[float]] = {
        "x": [math.sin(2 * math.pi * 0.5 * t) for t in timestamps],
        "y": [math.sin(2 * math.pi * 0.5 * t) for t in timestamps],
        "z": [math.sin(2 * math.pi * 0.5 * t) for t in timestamps],
    }

    # Create KRec with header
    start_time = int(time.time_ns())
    header = krec.KRecHeader(
        uuid=str(uuid.uuid4()),
        task="test-task",
        robot_platform="test-robot",
        robot_serial="test-serial",
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

    krec_obj = krec.KRec(header=header)

    # Add synthetic data using the correct API
    for i in range(num_frames):
        frame = krec.KRecFrame(
            video_timestamp=i,
            video_frame_number=i,
            inference_step=i,
            real_timestamp=i,
        )

        # Add actuator states for each joint
        for j in range(3):
            state = krec.ActuatorState(
                actuator_id=j,
                online=True,
                position=position_waves[j][i],
                velocity=velocity_waves[j][i],
                torque=torque_waves[j][i],
                temperature=25.0 + math.sin(2 * math.pi * 0.1 * timestamps[i]),  # Slowly varying temperature
                voltage=12.0 + math.sin(2 * math.pi * 0.05 * timestamps[i]),  # Slowly varying voltage
                current=1.0 + 0.1 * math.sin(2 * math.pi * 0.2 * timestamps[i]),  # Slowly varying current
            )
            frame.add_actuator_state(state)

        # Add IMU data
        imu = krec.IMUValues(
            accel=krec.Vec3(x=accel_waves["x"][i], y=accel_waves["y"][i], z=accel_waves["z"][i]),
            gyro=krec.Vec3(x=gyro_waves["x"][i], y=gyro_waves["y"][i], z=gyro_waves["z"][i]),
        )
        frame.set_imu_values(imu)

        krec_obj.add_frame(frame)

    original_data: dict[str, Mapping[int | str, list[float]]] = {
        "position_waves": position_waves,
        "velocity_waves": velocity_waves,
        "torque_waves": torque_waves,
        "accel_waves": accel_waves,
        "gyro_waves": gyro_waves,
        "timestamps": {"0": timestamps},
    }

    return krec_obj, original_data


def verify_krec_data(original_data: dict[str, Mapping[int | str, list[float]]], loaded_krec: krec.KRec) -> bool:
    """Verify that loaded KRec data matches the original data."""
    frames = loaded_krec.get_frames()

    def is_close(a: float | None, b: float, rtol: float = 1e-5) -> bool:
        if a is None:
            return False
        return abs(a - b) <= rtol * max(abs(a), abs(b))

    for i, frame in enumerate(frames):
        # Verify actuator states
        for j, state in enumerate(frame.get_actuator_states()):
            expected_pos = original_data["position_waves"][j][i]
            expected_vel = original_data["velocity_waves"][j][i]
            expected_torque = original_data["torque_waves"][j][i]

            if not is_close(state.position, expected_pos):
                logging.error(f"Position mismatch at frame {i}, joint {j}")
                return False
            if not is_close(state.velocity, expected_vel):
                logging.error(f"Velocity mismatch at frame {i}, joint {j}")
                return False
            if not is_close(state.torque, expected_torque):
                logging.error(f"Torque mismatch at frame {i}, joint {j}")
                return False

        # Verify IMU data
        imu = frame.get_imu_values()
        if imu and imu.accel and imu.gyro:
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

            for a, b in zip(actual_accel, expected_accel):
                if not is_close(a, b):
                    logging.error(f"Acceleration mismatch at frame {i}")
                    return False
            for a, b in zip(actual_gyro, expected_gyro):
                if not is_close(a, b):
                    logging.error(f"Gyro mismatch at frame {i}")
                    return False

    return True


def test_direct_save_load(
    synthetic_krec_data: tuple[krec.KRec, dict[str, Mapping[int | str, list[float]]]],
    tmpdir: Path,
) -> None:
    """Test saving and loading KRec data directly to/from a file."""
    krec_obj, original_data = synthetic_krec_data
    temp_file = tmpdir / "test.krec"
    krec_obj.save(str(temp_file))
    loaded_krec = krec.KRec.load(str(temp_file))
    assert verify_krec_data(original_data, loaded_krec)


def test_video_combination(
    synthetic_krec_data: tuple[krec.KRec, dict[str, Mapping[int | str, list[float]]]],
    tmpdir: Path,
) -> None:
    """Test combining KRec with video and extracting it back."""
    krec_obj, original_data = synthetic_krec_data

    # Create temporary file paths
    temp_krec = tmpdir / "temp.krec"
    temp_video = tmpdir / "temp.mkv"
    output_video = tmpdir / "combined.krec.mkv"

    # Save initial KRec
    krec_obj.save(str(temp_krec))

    # Create a dummy video file (1 second black screen)
    os.system(f"ffmpeg -f lavfi -i color=c=black:s=640x480:r=30 -t 1 {temp_video} -y")

    # Combine video with KRec
    krec.combine_with_video(str(temp_video), str(temp_krec), str(output_video))

    # Extract KRec from video
    extracted_krec = krec.extract_from_video(str(output_video), verbose=False)
    assert verify_krec_data(original_data, extracted_krec)


def test_header_preservation(
    synthetic_krec_data: tuple[krec.KRec, dict[str, dict[int | str, list[float]]]],
    tmpdir: Path,
) -> None:
    """Test that KRec header information is preserved during save/load."""
    krec_obj, _ = synthetic_krec_data

    temp_file = tmpdir / "test.krec"
    krec_obj.save(str(temp_file))
    loaded_krec = krec.KRec.load(str(temp_file))

    # Verify header fields
    assert krec_obj.header.uuid == loaded_krec.header.uuid
    assert krec_obj.header.task == loaded_krec.header.task
    assert krec_obj.header.robot_platform == loaded_krec.header.robot_platform
    assert krec_obj.header.robot_serial == loaded_krec.header.robot_serial
    assert krec_obj.header.start_timestamp == loaded_krec.header.start_timestamp
    assert krec_obj.header.end_timestamp == loaded_krec.header.end_timestamp

    # Verify actuator configs
    original_configs = krec_obj.header.get_actuator_configs()
    loaded_configs = loaded_krec.header.get_actuator_configs()
    assert len(original_configs) == len(loaded_configs)

    for orig_config, loaded_config in zip(original_configs, loaded_configs):
        assert orig_config.actuator_id == loaded_config.actuator_id
        assert orig_config.kp == loaded_config.kp
        assert orig_config.kd == loaded_config.kd
        assert orig_config.ki == loaded_config.ki
        assert orig_config.max_torque == loaded_config.max_torque
        assert orig_config.name == loaded_config.name


def test_invalid_file_handling(tmpdir: Path) -> None:
    """Test handling of invalid files and formats."""
    with pytest.raises(OSError):
        krec.KRec.load("nonexistent_file.krec")

    invalid_file = Path(tmpdir / "invalid.krec")
    invalid_file.write_bytes(b"invalid data")

    with pytest.raises(Exception):
        krec.KRec.load(str(invalid_file))
