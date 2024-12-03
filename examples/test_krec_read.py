import argparse
import krec
import os
from datetime import datetime

"""Usage:
python examples/test_krec_read.py --krec_file /path/to/krec/file

python examples/test_krec_read.py --krec_file /home/kasm-user/ali_repos/kmodel/data/datasets/krec_data/nov_29__8_37_pm_krec_w_mkv_w_states/recording_20241125_184810_c249e9f6-4ebf-48c7-b8ea-4aaad721a4f8_edited.krec.mkv
"""

def get_krec_file_type(file_path: str) -> str:
    """Determine if the file is a direct KREC file or MKV-embedded KREC.
    
    Returns:
        'krec' for .krec files
        'mkv' for .krec.mkv files
        raises RuntimeError for invalid extensions
    """
    if file_path.endswith('.krec'):
        return 'krec'
    elif file_path.endswith('.krec.mkv'):
        return 'mkv'
    else:
        error_msg = f"Invalid file extension. Expected '.krec' or '.krec.mkv', got: {file_path}"
        raise RuntimeError(error_msg)
    
def load_krec_direct(krec_file_path: str) -> krec.KRec:
    """Load a KREC file directly."""
    return krec.KRec.load(krec_file_path)

def load_krec_from_mkv(mkv_file_path: str) -> krec.KRec:
    """Load a KREC file from an MKV file into a manually created temp directory."""

    if not os.path.exists(mkv_file_path):
        raise FileNotFoundError(f"File not found: {mkv_file_path}")
    
    timestamp = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
    temp_dir = os.path.join(os.path.dirname(mkv_file_path), f"temp_{timestamp}")
    os.makedirs(temp_dir, exist_ok=True)
    
    base_name = os.path.basename(mkv_file_path).split('.krec.mkv')[0]
    krec_file_path = os.path.join(temp_dir, f"{base_name}_from_mkv.krec")
    
    # Extract and load from temp directory
    krec.extract_from_video(mkv_file_path, krec_file_path)
    # krec.extract_from_video(mkv_file_path, krec_file_path, verbose=True) # for getting the ffmpeg output
    return krec.KRec.load(krec_file_path)

def load_krec(file_path: str) -> krec.KRec:
    """Smart loader that handles both direct KREC and MKV-embedded KREC files."""
    file_type = get_krec_file_type(file_path)
    
    if file_type == 'krec':
        return load_krec_direct(file_path)
    else:  # file_type == 'mkv'
        return load_krec_from_mkv(file_path)


def main(args: argparse.Namespace) -> None:

    print(f"Reading KRec file: {args.krec_file}")
    krec_obj = load_krec(args.krec_file)
    print(f"krec_obj: {krec_obj}")
    print(f"krec_obj.header: {krec_obj.header}")
    print(f"num frames: {len(krec_obj)}")
    print(f"succesfully loaded KRec file!")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Read and display KRec file contents")
    parser.add_argument(
        "--krec_file",
        type=str,
        default="kdatagen/sim/resources/stompypro/krec_out/test_krec_write_out.krec",
        help="Path to KRec file to read",
    )
    args = parser.parse_args()
    main(args)