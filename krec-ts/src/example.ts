import { KRec } from "./krec";
import { promises as fs } from "fs";
import { Long } from "protobufjs/minimal";

// Helper function to convert Long timestamp to Date
function longToDate(timestamp: Long): Date {
  // Convert nanoseconds to milliseconds
  const ms = Number(timestamp) / 1_000_000;
  return new Date(ms);
}

// Helper function to format Long to number (safe for frame numbers)
function longToNumber(value: Long): number {
  return Number(value);
}

async function main() {
  const buffer = await fs.readFile("./test.krec");
  
  const krec = await KRec.load(buffer);
  
  console.log("Recording UUID:", krec.header.uuid);
  console.log("Number of frames:", krec.frames.length);
  
  if (krec.frames.length > 0) {
    const frame = krec.frames[0];
    console.log("First frame:", {
      timestamp: longToDate(frame.videoTimestamp as Long).toISOString(),
      frameNumber: longToNumber(frame.frameNumber as Long),
      actuatorStates: frame.actuatorStates?.map(state => ({
        online: state.online,
        position: state.position,
        velocity: state.velocity,
        torque: state.torque
      }))
    });
  }
}

main().catch(console.error); 