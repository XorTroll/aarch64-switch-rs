#!/usr/bin/env python3

import sys
import os
import struct
from enum import IntEnum
from datetime import datetime

class LogDataChunkKey(IntEnum):
    SessionBegin = 0
    SessionEnd = 1
    TextLog = 2
    LineNumber = 3
    FileName = 4
    FunctionName = 5
    ModuleName = 6
    ThreadName = 7
    LogPacketDropCount = 8
    UserSystemClock = 9
    ProcessName = 10

class LogSeverity(IntEnum):
    Trace = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
    Fatal = 4,

class LogPacketHeader:
        def __init__(self, process_id, thread_id, severity, verbosity, size):
            self.process_id = process_id
            self.thread_id = thread_id
            self.severity = severity
            self.verbosity = verbosity
            self.size = size

class LogDataChunk:
        def __init__(self, chunk_key, chunk_size, chunk_data):
            self.chunk_key = chunk_key
            self.chunk_size = chunk_size
            self.chunk_data = chunk_data

        def read_string(self):
            return str(self.chunk_data, "utf-8")

        def read_n(self, fmt):
            tpl = struct.unpack(fmt, self.chunk_data)
            return tpl[0]

        def read_u32(self):
            return self.read_n("I")

        def read_u64(self):
            return self.read_n("Q")

        def read_bool(self):
            return bool(self.read_n("B"))

        def format_data(self):
            if self.chunk_key is LogDataChunkKey.UserSystemClock:
                return str(self.read_u64())
            elif self.chunk_key is LogDataChunkKey.LineNumber:
                return str(self.read_u32())
            elif self.chunk_key is LogDataChunkKey.ThreadName:
                return self.read_string()
            elif self.chunk_key is LogDataChunkKey.FileName:
                return self.read_string()
            elif self.chunk_key is LogDataChunkKey.FunctionName:
                return self.read_string()
            elif self.chunk_key is LogDataChunkKey.FileName:
                return self.read_string()
            elif self.chunk_key is LogDataChunkKey.TextLog:
                return self.read_string()
            elif self.chunk_key is LogDataChunkKey.ProcessName:
                return self.read_string()
            elif self.chunk_key is LogDataChunkKey.ModuleName:
                return self.read_string()
            else:
                return "<unknown-data>"

def read_log_message(log_file):
    try:
        binlog_sz = os.stat(log_file).st_size
        if binlog_sz >= 0x18:
            with open(log_file, "rb") as binlog:
                header_data = binlog.read(0x18)
                (process_id, thread_id, _pad, severity, verbosity, payload_size) = struct.unpack("QQHBBI", header_data)
                offset = 0x18
                sth_printed = False
                print("LogPacket {")
                while offset < payload_size:
                    chunk_info = binlog.read(2)
                    offset += 2
                    chunk_key = LogDataChunkKey(int(chunk_info[0]))
                    chunk_size = int(chunk_info[1])
                    chunk_data = binlog.read(chunk_size)
                    chunk = LogDataChunk(chunk_key, chunk_size, chunk_data)
                    chunk_str = chunk.format_data()
                    offset += chunk_size
                        
                    print("  " + str(chunk_key) + ": '" + chunk_str + "',")
                    sth_printed = True
                print("}\n")
                    
    except Exception as err:
        print("Error with " + log_file + ": " + str(err))

def main():
    print("Log reader")
    print()
    if len(sys.argv) >= 1:
        logs = []
        for i in range(len(sys.argv) - 1):
            logs_dir = sys.argv[i + 1]
            for (dirpath, dirnames, filenames) in os.walk(logs_dir):
                for file in filenames:
                    try:
                        tickf = file[:-4]
                        tick_v = int(tickf, 16)
                        logs.append((os.path.join(logs_dir, file), tick_v))
                    except Exception as err:
                        pass
        logs.sort(key = lambda x: x[1])
        for log in logs:
            read_log_message(log[0])

if __name__ == "__main__":
    main()