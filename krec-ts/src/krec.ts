import { Reader } from "protobufjs";
import { krec } from "./generated/proto";

export class KRec {
  header: krec.proto.IKRecHeader;
  frames: krec.proto.IKRecFrame[];

  constructor(header: krec.proto.IKRecHeader) {
    this.header = header;
    this.frames = [];
  }

  static async load(buffer: Buffer): Promise<KRec> {
    let pos = 0;

    // Read header length and decode header
    if (buffer.length < 4) {
      throw new Error(`File too short: ${buffer.length} bytes`);
    }
    
    const headerLen = buffer.readUInt32LE(0);
    pos += 4;

    if (pos + headerLen > buffer.length) {
      throw new Error(
        `Incomplete header data: need ${headerLen} bytes, have ${buffer.length - pos} bytes`
      );
    }

    const header = krec.proto.KRecHeader.decode(
      Reader.create(buffer.subarray(pos, pos + headerLen))
    );
    pos += headerLen;

    const frames: krec.proto.IKRecFrame[] = [];

    // Read frames
    while (pos + 4 <= buffer.length) {
      const frameLen = buffer.readUInt32LE(pos);
      pos += 4;

      if (pos + frameLen > buffer.length) {
        throw new Error(
          `Incomplete frame data: at position ${pos}, need ${frameLen} bytes, have ${
            buffer.length - pos
          } bytes remaining`
        );
      }

      const frame = krec.proto.KRecFrame.decode(
        Reader.create(buffer.subarray(pos, pos + frameLen))
      );
      pos += frameLen;
      frames.push(frame);
    }

    if (pos !== buffer.length) {
      throw new Error(
        `Trailing data: ${buffer.length - pos} bytes remaining after position ${pos}`
      );
    }

    const krecInstance = new KRec(header);
    krecInstance.frames = frames;
    return krecInstance;
  }
} 