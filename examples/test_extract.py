"""Extract KRec data from an MKV file and optionally save it to a new file.

Usage:
# Extract and view
python examples/test_extract.py input.mkv -v

# Extract and save
python examples/test_extract.py input.mkv -o output.krec -v
"""

import argparse
import logging
import sys
from pathlib import Path

import colorlogging

import krec


def main() -> int:
    parser = argparse.ArgumentParser(description="Extract KRec data from MKV file")
    parser.add_argument("input_file", type=str, help="Input MKV file path")
    parser.add_argument("-o", "--output-file", type=str, help="Output KRec file path (optional)")
    parser.add_argument("-v", "--verbose", action="store_true", help="Enable verbose output")

    colorlogging.configure()
    args = parser.parse_args()

    logging.info("Extracting from: %s", args.input_file)

    try:
        extracted_krec = krec.extract_from_video(args.input_file, verbose=args.verbose)
        logging.info("Extraction successful")
        logging.info("Extracted KRec has %d frames", len(extracted_krec))

        if args.output_file:
            output_path = Path(args.output_file)
            extracted_krec.save(str(output_path))
            logging.info("Saved to: %s", output_path)

    except Exception as e:
        logging.error("Error: %s", e)
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
