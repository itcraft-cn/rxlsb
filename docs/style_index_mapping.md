# Excel XLSB格式(styleIndex)映射机制详解

## 问题背景

在实现XLSB写入器时，cell.styleIndex的正确映射是一个容易出错的关键问题。

**错误理解**：
- cell.styleIndex是全局XF索引（XF[0], XF[1], XF[2]...）
- 需要手动+1映射：styles[i] → XF[i+1]

**正确理解**：
- cell.styleIndex是styles列表索引（0-based）
- Excel/WPS读取后自动+1映射到全局XF索引

---

## XF结构说明

### BrtBeginXFs vs BrtBeginStyles

**XF全局索引结构**：
```
XF[0]: BrtBeginXFs (cell XF, ixf=0xffff)
       用于普通单元格，无特殊格式
       
XF[1+]: BrtBeginStyles (style XF, ixf=0x0000)
       用于带格式的单元格
       styles列表对应这部分
```

### styles列表与XF的映射关系

```
styles列表    全局XF索引
styles[0]  →  XF[1]  (BrtBeginStyles[0])
styles[1]  →  XF[2]  (BrtBeginStyles[1])
styles[2]  →  XF[3]  (BrtBeginStyles[2])
styles[i]  →  XF[i+1] (自动+1映射)
```

**关键点**：
- XF[0]不在styles列表中
- styles列表从XF[1]开始
- Excel/WPS自动处理+1偏移

---

## 实现要点

### 1. styles注册函数返回值

**正确实现**：
```rust
// Rust (rxlsb)
pub fn get_style_id_for_format(&mut self, format_string: &str) -> u32 {
    let format_id = self.format_registry.get_or_add_format(format_string);
    
    // 查找已存在的style
    for (i, xf) in self.style_xfs.iter().enumerate() {
        if xf.num_fmt_id == format_id {
            return i as u32;  // 返回styles列表索引
        }
    }
    
    // 创建新style
    self.style_xfs.push(StyleXF { num_fmt_id: format_id, ... });
    return (self.style_xfs.len() - 1) as u32;  // 返回styles列表索引
}
```

```c
// C (cxlsb)
int cxlsb_styles_registry_add_style(cxlsb_styles_registry_t* registry, int num_fmt_id) {
    // 查找已存在的style
    for (uint32_t i = 0; i < registry->style_count; i++) {
        if (registry->styles[i].num_fmt_id == num_fmt_id) {
            return i;  // 返回styles列表索引 ✓ 正确
        }
    }
    
    // 创建新style
    registry->styles[registry->style_count].num_fmt_id = num_fmt_id;
    return registry->style_count++;  // 返回styles列表索引 ✓ 正确
}
```

**错误实现**（不要这样写）：
```rust
// ✗ 错误：手动+1
return (i + 1) as u32;  // 会导致错位
return self.style_xfs.len() as u32;  // 会导致错位
```

### 2. 写入cell.styleIndex

**写入方式**：
```rust
// 直接写入styles列表索引
write_cell_real_with_style(row, col, value, style_idx);  // style_idx是styles索引
```

**字节序列**：
```
cell.styleIndex字段（4字节uint32，小端序）：
styleIndex=1 → 01 00 00 00
styleIndex=2 → 02 00 00 00
styleIndex=3 → 03 00 00 00
```

**Excel/WPS读取后**：
```
读取styleIndex=1 → 自动+1 → 使用XF[2]
读取styleIndex=2 → 自动+1 → 使用XF[3]
读取styleIndex=3 → 自动+1 → 使用XF[4]
```

---

## 实际案例验证

### 案例：format_test2.xlsb

**styles注册顺序**：
```
SheetWriter构造:
  get_style_id_for_format("m/d/yy h:mm") → ifmt=22
  styles[1] = {numFmtId: 22} → 返回1

Col 0 (date):
  get_style_id_for_format("m/d/yy h:mm") → ifmt=22
  找到styles[1] → 返回1

Col 1 (time):
  get_style_id_for_format("h:mm:ss") → ifmt=21
  styles[2] = {numFmtId: 21} → 返回2

Col 2 (currency):
  get_style_id_for_format("￥#,##0.00") → ifmt=164
  styles[3] = {numFmtId: 164} → 返回3

Col 3 (negRed):
  get_style_id_for_format("#,##0.00;[Red]-#,##0.00") → ifmt=165
  styles[4] = {numFmtId: 165} → 返回4
```

**XF ifmt顺序**：
```
XF[0]: ifmt=0   (cell XF, BrtBeginXFs)
XF[1]: ifmt=0   (style XF, styles[0], default)
XF[2]: ifmt=22  (style XF, styles[1], date)
XF[3]: ifmt=21  (style XF, styles[2], time)
XF[4]: ifmt=164 (style XF, styles[3], currency)
XF[5]: ifmt=165 (style XF, styles[4], negRed)
```

**cell.styleIndex映射验证**：
```
Col 0: 写入styleIndex=1 → Excel/WPS +1 → XF[2] (ifmt=22, date) ✓
Col 1: 写入styleIndex=2 → Excel/WPS +1 → XF[3] (ifmt=21, time) ✓
Col 2: 写入styleIndex=3 → Excel/WPS +1 → XF[4] (ifmt=164, currency) ✓
Col 3: 写入styleIndex=4 → Excel/WPS +1 → XF[5] (ifmt=165, negRed) ✓
```

---

## 错误案例分析

### 第一次错误（rxlsb）

**错误代码**：
```rust
pub fn get_style_id_for_format(&mut self, format_string: &str) -> u32 {
    for (i, xf) in self.style_xfs.iter().enumerate() {
        if xf.num_fmt_id == format_id {
            return (i + 1) as u32;  // ✗ 错误：手动+1
        }
    }
    return self.style_xfs.len() as u32;  // ✗ 错误：手动+1
}
```

**结果**：
```
写入styleIndex=2 → Excel/WPS +1 → XF[3]
期望XF[2] (ifmt=22)，实际XF[3] (ifmt=21)
→ 所有列错位一位！
```

**用户反馈**：
```
"要求：第一列日期，第二列时间，第三列金额，第四列数值（有负数，标红）
 实际：第一列时间，第二列人民币符号+4xxxxx.yy，第三列数值，负红，第四列数值
 感觉是都错位了一位"
```

### 第二次修正（正确）

**正确代码**：
```rust
pub fn get_style_id_for_format(&mut self, format_string: &str) -> u32 {
    for (i, xf) in self.style_xfs.iter().enumerate() {
        if xf.num_fmt_id == format_id {
            return i as u32;  // ✓ 正确：返回styles索引
        }
    }
    return (self.style_xfs.len() - 1) as u32;  // ✓ 正确：返回styles索引
}
```

**结果**：
```
写入styleIndex=1 → Excel/WPS +1 → XF[2] (ifmt=22, date) ✓
所有列显示正确！
```

---

## cxlsb检查结果

**cxlsb当前实现**（src/format/styles_registry.c）：
```c
int cxlsb_styles_registry_add_style(cxlsb_styles_registry_t* registry, int num_fmt_id) {
    for (uint32_t i = 0; i < registry->style_count; i++) {
        if (registry->styles[i].num_fmt_id == num_fmt_id) {
            return i;  // ✓ 正确：返回styles列表索引
        }
    }
    
    registry->styles[registry->style_count].num_fmt_id = num_fmt_id;
    return registry->style_count++;  // ✓ 正确：返回styles列表索引
}
```

**结论**：cxlsb的实现是正确的！不需要修改。

---

## 核心要点总结

1. **cell.styleIndex是styles列表索引（0-based）**
   - 不是全局XF索引
   - 不需要手动+1

2. **Excel/WPS自动+1映射**
   - styles[i] → XF[i+1]
   - 因为XF[0]在BrtBeginXFs，XF[1+]在BrtBeginStyles

3. **get_style_id_for_format返回值**
   - 返回styles列表索引i
   - 不要返回i+1（会导致错位）

4. **写入cell时**
   - 直接写入styles列表索引
   - Excel/WPS会自动处理偏移

5. **验证方法**
   - 检查XF ifmt顺序
   - 验证styleIndex映射
   - 在Excel/WPS中打开确认显示

---

## 参考文件

- rxlsb: `/disk2/helly_data/code/rust/rxlsb/src/format/styles_registry.rs`
- cxlsb: `/disk2/helly_data/code/ansi_c/cxlsb/src/format/styles_registry.c`
- 测试: `/disk2/helly_data/code/rust/rxlsb/examples/format_test2.rs`

---

## 相关知识

- MS-XLSB规范：[MS-XLSB] BrtXF结构
- Excel内置格式ID：0-22
- 自定义格式ID起始：164
- BrtCellReal结构：cell(8字节) + xnum(8字节)
- cell结构：column(4字节) + styleIndex(4字节)