#!/usr/bin/env python3
"""
验证XLSB文件的styleIndex映射是否正确
用法: python3 verify_style_index.py <xlsb_file>
"""

import struct
import zipfile
import sys

def decode_varint(data, offset):
    result = 0
    shift = 0
    i = offset
    while i < len(data):
        b = data[i]
        result |= (b & 0x7F) << shift
        i += 1
        if (b & 0x80) == 0:
            break
        shift += 7
    return result, i

def verify_style_index(xlsb_path):
    print("=" * 70)
    print(f"验证文件: {xlsb_path}")
    print("=" * 70)
    
    with zipfile.ZipFile(xlsb_path, 'r') as z:
        styles = z.read('xl/styles.bin')
        sheet = z.read('xl/worksheets/sheet1.bin')
    
    # 解析XF
    xf_list = []
    offset = 0
    
    while offset < len(styles):
        rec_type, n1 = decode_varint(styles, offset)
        rec_size, n2 = decode_varint(styles, n1)
        
        if rec_type in [626, 627, 617, 618]:
            if rec_type == 626 or rec_type == 617:
                offset = n2 + 4
            else:
                offset = n2
            continue
        
        if rec_type == 47:
            xf_bytes = styles[n2:n2+rec_size]
            ixf = struct.unpack('<H', xf_bytes[0:2])[0]
            ifmt = struct.unpack('<H', xf_bytes[2:4])[0]
            xf_type = "cell XF" if ixf == 0xffff else "style XF"
            xf_list.append({'ixf': ixf, 'ifmt': ifmt, 'type': xf_type})
        
        offset = n2 + rec_size
    
    print(f"\nXF总数: {len(xf_list)}")
    print("XF结构:")
    
    format_names = {
        0: "General",
        14: "mm-dd-yy",
        21: "h:mm:ss",
        22: "m/d/yy h:mm",
    }
    
    for i, xf in enumerate(xf_list):
        fmt_name = format_names.get(xf['ifmt'], f"ifmt={xf['ifmt']}")
        if xf['ifmt'] >= 164:
            fmt_name = f"自定义格式(ifmt={xf['ifmt']})"
        print(f"  XF[{i}]: {xf['type']}, {fmt_name}")
    
    # 解析Cells
    cells = []
    offset = 0
    
    while offset < len(sheet):
        rec_type, n1 = decode_varint(sheet, offset)
        rec_size, n2 = decode_varint(sheet, n1)
        
        if rec_type == 5:  # BrtCellReal
            col = struct.unpack('<I', sheet[n2:n2+4])[0]
            style_idx = struct.unpack('<I', sheet[n2+4:n2+8])[0]
            value = struct.unpack('<d', sheet[n2+8:n2+16])[0]
            
            if offset < 400:  # Row 0
                cells.append((col, style_idx, value))
        
        offset = n2 + rec_size
    
    print("\n" + "=" * 70)
    print("Row 0 Cells映射验证")
    print("=" * 70)
    
    print("\n映射机制:")
    print("  cell.styleIndex = styles列表索引")
    print("  Excel/WPS +1 → XF全局索引")
    print("  styles[i] → XF[i+1]")
    
    print("\n实际映射:")
    for col, style_idx, value in sorted(cells[:min(8, len(cells))]):
        xf_idx_written = style_idx
        xf_idx_display = style_idx + 1
        
        xf_written = xf_list[xf_idx_written] if xf_idx_written < len(xf_list) else None
        xf_display = xf_list[xf_idx_display] if xf_idx_display < len(xf_list) else None
        
        print(f"\nCol {col}:")
        print(f"  数据值: {value:.2f}")
        print(f"  写入styleIndex: {style_idx} (styles列表索引)")
        print(f"  → 写入对应XF[{xf_idx_written}]")
        
        if xf_written:
            fmt_written = format_names.get(xf_written['ifmt'], f"ifmt={xf_written['ifmt']}")
            print(f"     ({xf_written['type']}, {fmt_written})")
        
        print(f"  Excel/WPS +1 → XF[{xf_idx_display}]")
        
        if xf_display:
            fmt_display = format_names.get(xf_display['ifmt'], f"ifmt={xf_display['ifmt']}")
            print(f"     ({xf_display['type']}, {fmt_display})")
            print(f"  ✓ 实际显示格式: {fmt_display}")
    
    print("\n" + "=" * 70)
    print("验证结论")
    print("=" * 70)
    
    print("\n关键点:")
    print("1. styles注册函数返回styles列表索引（不要+1）")
    print("2. 写入cell时直接使用styles索引")
    print("3. Excel/WPS自动+1映射到XF全局索引")
    print("4. styles[i] → XF[i+1]（因为XF[0]在BrtBeginXFs）")
    
    print("\n✓ 如果所有列显示格式正确，说明映射机制实现正确")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("用法: python3 verify_style_index.py <xlsb_file>")
        print("示例: python3 verify_style_index.py /tmp/format_test2.xlsb")
        sys.exit(1)
    
    verify_style_index(sys.argv[1])