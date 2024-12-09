"""Script to read and display KRec file contents.

Usage:
python examples/test_krec_read.py --krec_file /path/to/krec/file.krec

python examples/test_krec_read.py --krec_file /path/to/krec/file.krec.mkv

python examples/test_krec_read.py --krec_file /path/to/krec/file.krec.mkv -v
"""

import argparse
import logging
import os
import sys
import traceback

import colorlogging

import krec


def get_krec_file_type(file_path: str) -> str:
    """Determine if the file is a direct KREC file or MKV-embedded KREC.

    Returns:
        'krec' for .krec files
        'mkv' for .krec.mkv files
        raises RuntimeError for invalid extensions
    """
    if file_path.endswith(".krec"):
        return "krec"
    elif file_path.endswith(".krec.mkv"):
        return "mkv"
    else:
        error_msg = f"Invalid file extension. Expected '.krec' or '.krec.mkv', got: {file_path}"
        raise RuntimeError(error_msg)


def load_krec_direct(krec_file_path: str) -> krec.KRec:
    """Load a KREC file directly."""
    return krec.KRec.load(krec_file_path)


def load_krec_from_mkv(mkv_file_path: str, verbose: bool) -> krec.KRec:
    """Load a KREC file from an MKV file into a manually created temp directory."""
    if not os.path.exists(mkv_file_path):
        raise FileNotFoundError(f"File not found: {mkv_file_path}")

    return krec.extract_from_video(mkv_file_path, verbose=verbose)


def load_krec(file_path: str, verbose: bool) -> krec.KRec:
    """Smart loader that handles both direct KREC and MKV-embedded KREC files."""
    file_type = get_krec_file_type(file_path)
    return load_krec_direct(file_path) if file_type == "krec" else load_krec_from_mkv(file_path, verbose)


def main() -> int:
    parser = argparse.ArgumentParser(description="Read and display KRec file contents")
    parser.add_argument(
        "--krec_file",
        type=str,
        default="kdatagen/sim/resources/stompypro/krec_out/test_krec_write_out.krec",
        help="Path to KRec file to read",
    )
    parser.add_argument("-v", "--verbose", action="store_true", help="Enable verbose output for ffmpeg")

    colorlogging.configure()
    args = parser.parse_args()

    try:
        logging.info("Reading KRec file: %s", args.krec_file)
        krec_obj = load_krec(args.krec_file, verbose=args.verbose)
        logging.info("KRec object: %s", krec_obj)
        logging.info("KRec header: %s", krec_obj.header)
        logging.info("Number of frames: %d", len(krec_obj))
        logging.info("Successfully loaded KRec file!")
        return 0
    except Exception as e:
        logging.error("Error: %s", e)
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())
