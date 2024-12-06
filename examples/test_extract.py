import krec
import argparse
from pathlib import Path


"""
# Just extract and view
python examples/test_extract.py input.mkv -v

# Extract and save
python examples/test_extract.py input.mkv -o output.krec -v
"""


def main(args):
    print(f"Extracting from: {args.input_file}")

    try:
        extracted_krec = krec.extract_from_video(args.input_file, verbose=args.verbose)
        print("Extraction successful")
        print(f"Extracted KRec has {len(extracted_krec)} frames")

        if args.output_file:
            output_path = Path(args.output_file)
            extracted_krec.save(str(output_path))
            print(f"Saved to: {output_path}")

    except Exception as e:
        print(f"Error: {e}")
        return 1

    return 0


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Extract KRec data from MKV file")
    parser.add_argument("input_file", type=str, help="Input MKV file path")
    parser.add_argument("-o", "--output-file", type=str, help="Output KRec file path (optional)")
    parser.add_argument("-v", "--verbose", action="store_true", help="Enable verbose output")

    args = parser.parse_args()
    exit(main(args))
