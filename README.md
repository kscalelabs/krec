# kclip

K-Clips are the way that we collect data to train our AI models.

## Append metadata to a video file

### FFmpeg

`ffmpeg -i test_video.mp4 -attach test.txt -metadata:s:t mimetype=application/octet-stream -metadata:s:t title="kclip-001" -metadata:s:t uuid="123e4567-e89b-12d3-a456-426614174000" -metadata:s:t action="put the bunny back in the box" -c copy output_kclip.mkv`

## Read metadata

### JS

[ffmpeg-wasm](adhoc/web/full_meta.html)
