"""Usage:
# Visualize synthetic KRec data:
python examples/krec_to_rrd.py --synthetic --spawn-viewer -v
python examples/krec_to_rrd.py --input /home/kasm-user/ali_repos/kmodel/data/datasets/krec_data/dec_3__11_10am_og_krecs_edited/2024-12-03_17-47-30/recording_20241125_184919_04bf0bae-c5d7-46bb-b1ef-e021c1ad85f8.krec_edited.krec.mkv --spawn-viewer -v
python examples/krec_to_rrd.py --input /home/kasm-user/ali_repos/kmodel/data/datasets/krec_data/dec_3__11_10am_og_krecs_edited/2024-12-03_17-47-30/temp/temp_2024-12-04_18-25-38/recording_20241125_184841_aad3f390-2589-4fb7-a4b1-c9fab8dd8cc8.krec_edited_from_mkv.krec --spawn-viewer -v
python examples/krec_to_rrd.py --input /home/kasm-user/ali_repos/kmodel/data/datasets/krec_data/dec_3__11_10am_og_krecs/recording_20241125_184810_c249e9f6-4ebf-48c7-b8ea-4aaad721a4f8.krec.mkv --spawn-viewer -v

python examples/krec_to_rrd.py --input /home/kasm-user/ali_repos/kmodel/data/datasets/krec_data/dec_3__11_10am_og_krecs/recording_20241125_184810_c249e9f6-4ebf-48c7-b8ea-4aaad721a4f8.krec.mkv --spawn-viewer -v


# Save to RRD file:
python examples/krec_to_rrd.py --synthetic --output output.rrd -v
"""

import argparse
import logging
import time
from pathlib import Path

import krec
import rerun as rr
import rerun.blueprint as rrb

from generate_synthetic_krec import generate_synthetic_krec


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
        prefix = f"actuators/actuator_{state.actuator_id}/state"
        rr.log(f"{prefix}/position", rr.Scalar(state.position))
        rr.log(f"{prefix}/velocity", rr.Scalar(state.velocity))
        rr.log(f"{prefix}/torque", rr.Scalar(state.torque))
        rr.log(f"{prefix}/temperature", rr.Scalar(state.temperature))
        rr.log(f"{prefix}/voltage", rr.Scalar(state.voltage))
        rr.log(f"{prefix}/current", rr.Scalar(state.current))
        rr.log(f"{prefix}/online", rr.Scalar(float(state.online)))

    # Log actuator commands
    for cmd in frame.get_actuator_commands():
        prefix = f"actuators/actuator_{cmd.actuator_id}/command"
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


def get_grid_blueprint():
    """Returns a blueprint with a 2x5 grid layout of actuator data."""
    return rrb.Vertical(
        # Header info at top
        rrb.TextDocumentView(name="Metadata", contents=["+ /krec/header/**"]),
        # Two rows of 5 actuators each
        rrb.Vertical(
            rrb.Horizontal(  # First row of 5
                contents=[
                    rrb.TimeSeriesView(
                        name=f"Actuator {i}",
                        contents=[f"+ /actuators/actuator_{i}/**"],
                        axis_y=rrb.ScalarAxis(range=(-2.0, 2.0)),  # Slightly larger than ±1 for sine waves
                    )
                    for i in range(5)  # Actuators 0-4
                ],
            ),
            rrb.Horizontal(  # Second row of 5
                contents=[
                    rrb.TimeSeriesView(
                        name=f"Actuator {i}",
                        contents=[f"+ /actuators/actuator_{i}/**"],
                        axis_y=rrb.ScalarAxis(range=(-2.0, 2.0)),
                    )
                    for i in range(5, 10)  # Actuators 5-9
                ],
            ),
        ),
        row_shares=[1, 8],
        name="Grid View",
    )


def get_grouped_blueprint():
    """Returns a blueprint with actuator data grouped by measurement type."""
    return rrb.Vertical(
        # Header info at top
        rrb.TextDocumentView(name="Metadata", contents=["+ /krec/header/**"]),
        # Main content area split into left and right
        rrb.Horizontal(
            # Left side: Commands and States
            rrb.Vertical(
                contents=[
                    rrb.TimeSeriesView(
                        name="Position Commands",
                        contents=[f"+ /actuators/actuator_{i}/command/position" for i in range(10)],
                        axis_y=rrb.ScalarAxis(range=(-2.0, 2.0)),
                    ),
                    rrb.TimeSeriesView(
                        name="Position States",
                        contents=[f"+ /actuators/actuator_{i}/state/position" for i in range(10)],
                        axis_y=rrb.ScalarAxis(range=(-2.0, 2.0)),
                    ),
                    rrb.TimeSeriesView(
                        name="Velocity Commands",
                        contents=[f"+ /actuators/actuator_{i}/command/velocity" for i in range(10)],
                        axis_y=rrb.ScalarAxis(range=(-2.0, 2.0)),
                    ),
                    rrb.TimeSeriesView(
                        name="Velocity States",
                        contents=[f"+ /actuators/actuator_{i}/state/velocity" for i in range(10)],
                        axis_y=rrb.ScalarAxis(range=(-2.0, 2.0)),
                    ),
                ]
            ),
            # Right side: Other measurements
            rrb.Vertical(
                contents=[
                    rrb.TimeSeriesView(
                        name="Torque Commands",
                        contents=[f"+ /actuators/actuator_{i}/command/torque" for i in range(10)],
                        axis_y=rrb.ScalarAxis(range=(-2.0, 2.0)),
                    ),
                    rrb.TimeSeriesView(
                        name="Torque States",
                        contents=[f"+ /actuators/actuator_{i}/state/torque" for i in range(10)],
                        axis_y=rrb.ScalarAxis(range=(-2.0, 2.0)),
                    ),
                    rrb.TimeSeriesView(
                        name="Temperature",
                        contents=[f"+ /actuators/actuator_{i}/state/temperature" for i in range(10)],
                        axis_y=rrb.ScalarAxis(range=(0.0, 50.0)),  # Temperature typically 20-30°C
                    ),
                    rrb.TimeSeriesView(
                        name="Voltage",
                        contents=[f"+ /actuators/actuator_{i}/state/voltage" for i in range(10)],
                        axis_y=rrb.ScalarAxis(range=(0.0, 15.0)),  # Voltage typically around 12V
                    ),
                    rrb.TimeSeriesView(
                        name="Current",
                        contents=[f"+ /actuators/actuator_{i}/state/current" for i in range(10)],
                        axis_y=rrb.ScalarAxis(range=(0.0, 2.0)),  # Current typically around 1A
                    ),
                ]
            ),
        ),
        row_shares=[1, 8],  # Header takes 1/9 of space, main content takes 8/9
        name="Grouped View",
    )


def get_combined_blueprint():
    """Returns a blueprint with both grouped and grid views in separate tabs."""
    return rrb.Blueprint(
        rrb.Tabs(
            get_grouped_blueprint(),
            get_grid_blueprint(),
            active_tab=0,  # Start with the grouped view
        ),
        rrb.BlueprintPanel(state="expanded"),
        rrb.SelectionPanel(state="collapsed"),
        rrb.TimePanel(state="expanded"),
    )


def main(args):
    logging.info("Starting visualization process")

    try:
        # Initialize Rerun
        spawn_viewer = args.spawn_viewer and not args.output
        rr.init("krec_visualization", spawn=spawn_viewer)
        rr.send_blueprint(get_combined_blueprint())

        if args.synthetic:
            extracted_krec = generate_synthetic_krec()
            logging.info("Created synthetic KRec data")
        else:
            extracted_krec = krec.extract_from_video(args.input, verbose=args.verbose)

        logging.info("Processing %d frames" % len(extracted_krec))

        # Log header information
        header = extracted_krec.header

        # Log each frame
        for idx, frame in enumerate(extracted_krec):
            log_frame_data(frame, idx)

        rr.log(
            "/krec/header",
            rr.TextDocument(
                f"UUID: {header.uuid}\n"
                f"Task: {header.task}\n"
                f"Robot Platform: {header.robot_platform}\n"
                f"Robot Serial: {header.robot_serial}\n"
                f"Start Timestamp: {header.start_timestamp}\n"
                f"End Timestamp: {header.end_timestamp}",
                media_type=rr.MediaType.MARKDOWN,
            ),
        )
        logging.info("Logged header information")

        # Save to RRD file if output path provided
        if args.output:
            output_path = Path(args.output)
            rr.save(str(output_path))
            logging.info("Saved RRD to: %s" % output_path)
        elif args.spawn_viewer:
            try:
                while True:
                    time.sleep(1)
            except KeyboardInterrupt:
                logging.info("Ctrl-C received. Exiting.")

    except Exception as e:
        logging.error("Error: %s" % e)
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
