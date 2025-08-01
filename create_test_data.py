#!/usr/bin/env python3
"""
Create test binary data that matches shairport-sync metadata format
"""

import struct

def create_metadata_item(item_type, code, data):
    """Create a binary metadata item"""
    # Convert strings to 4-byte values
    item_type_bytes = item_type.encode('ascii')[:4].ljust(4, b'\x00')
    code_bytes = code.encode('ascii')[:4].ljust(4, b'\x00')
    
    # Length as big-endian uint32
    length = len(data)
    length_bytes = struct.pack('>I', length)
    
    return item_type_bytes + code_bytes + length_bytes + data

# Create test data
test_items = []

# Title metadata
title_data = "Hello World".encode('utf-8')
test_items.append(create_metadata_item("core", "minm", title_data))

# Artist metadata  
artist_data = "Test Artist".encode('utf-8')
test_items.append(create_metadata_item("core", "asar", artist_data))

# Play begin (no data)
test_items.append(create_metadata_item("ssnc", "pbeg", b""))

# Play end (no data)
test_items.append(create_metadata_item("ssnc", "pend", b""))

# Write to file
with open('test_metadata.bin', 'wb') as f:
    for item in test_items:
        f.write(item)

print("Created test_metadata.bin with {} items".format(len(test_items)))
print("File size: {} bytes".format(sum(len(item) for item in test_items)))

# Also create a hex dump for inspection
with open('test_metadata.bin', 'rb') as f:
    data = f.read()
    
print("\nHex dump:")
for i in range(0, len(data), 16):
    chunk = data[i:i+16]
    hex_str = ' '.join(f'{b:02x}' for b in chunk)
    ascii_str = ''.join(chr(b) if 32 <= b <= 126 else '.' for b in chunk)
    print(f"{i:04x}: {hex_str:<48} {ascii_str}")