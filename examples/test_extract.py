"""
Extract KRec data from an MKV file and optionally save it to a new file.

Usage:
# Just extract and view
python examples/test_extract.py input.mkv -v

# Extract and save
python examples/test_extract.py input.mkv -o output.krec -v
"""

import argparse
import logging
from pathlib import Path

import krec


def main(args):
    logging.info(f"Extracting from: {args.input_file}")

    try:
        extracted_krec = krec.extract_from_video(args.input_file, verbose=args.verbose)
        logging.info("Extraction successful")
        logging.info(f"Extracted KRec has {len(extracted_krec)} frames")

        if args.output_file:
            output_path = Path(args.output_file)
            extracted_krec.save(str(output_path))
            logging.info(f"Saved to: {output_path}")

    except Exception as e:
        logging.error(f"Error: {e}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Extract KRec data from MKV file")
    parser.add_argument("input_file", type=str, help="Input MKV file path")
    parser.add_argument("-o", "--output-file", type=str, help="Output KRec file path (optional)")
    parser.add_argument("-v", "--verbose", action="store_true", help="Enable verbose output")

    logging.basicConfig(level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s")

    args = parser.parse_args()
    main(args)
