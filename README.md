# KiwiXML Compiler

**KiwiXML Compiler** is a cross‑platform XML processor built to simplify working with Blade & Soul XMLs

---

## 🧠 What It Does

The compiler’s role is to:

1. Read the base XML files (e.g., `0_KFM_Wolf_New.xml`).
2. Automatically expand any `<!-- #include file="..." -->` statements it finds.
3. Recursively insert the referenced XML content.
4. Clean up commented lines and format placeholder content into CDATA blocks.
5. Output a compiled, ready‑to‑use version into `./Compiled/`.

This modular structure allows you to focus on clean, organized, and maintainable XMLs rather than manually managing
large CDATA blocks.

---

## ⚙️ Include System

Instead of embedding raw data inside CDATA sections, you can reference other XML files with `#include` directives.

### Example

**Base XML (`KFM/0_KFM_Wolf_New.xml`):**

```xml

<patches>
    <patch file="skill3_contextscriptdata_kungfufighter_contextsimplemode_g3.xml">
        <!-- Include modular stance patches -->
        <!-- #include file="Wolf\단타.xml" -->
        <!-- #include file="Wolf\연계.xml" -->
    </patch>
</patches>
```

**Included File (`Wolf\단타.xml`):**

```xml

<placeholder>
    <!-- Base stance files -->
    <!-- #include file="Titan\Finishers.xml" -->
    <!-- #include file="Human.xml" -->
    <!-- #include file="Wolf.xml" -->
</placeholder>
```

When the compiler runs, it reads `0_KFM_Wolf_New.xml`, follows all `#include` references, and inlines their content
recursively — producing a complete, patched XML file.

✅ **Recursive support** — includes can reference other includes at any depth.  
✅ **Cross‑platform** — Windows‑style paths (e.g., `Wolf\단타.xml`) work seamlessly on both Linux and Windows.

---

## 🧩 Folder Structure

The compiler expects this clean, modular directory layout:

```
.
├── Compiled/
│   ├── 0_KFM_Wolf_New.xml
│   ├── 0_SF_Earth.xml
│   ├── 0_SF_Ice.xml
│   ├── 9_General.xml
│   └── 9_WL_Escape_To_B.xml
│
├── KFM/
│   ├── 0_KFM_Wolf_New.xml
│   ├── 9_General.xml
│   ├── General/
│   │   ├── Auto Extend.xml
│   │   └── Dead Skill Use.xml
│   └── Wolf/
│       ├── Human.xml
│       ├── Titan/
│       │   ├── Finishers.xml
│       │   ├── First.xml
│       │   ├── Second.xml
│       │   └── Third.xml
│       ├── Wolf.xml
│       ├── 단타.xml
│       └── 연계.xml
│
├── SF/
│   ├── 0_SF_Earth.xml
│   ├── 0_SF_Ice.xml
│   ├── Earth/
│   │   └── Melee.xml
│   └── Ice/
│       ├── Melee.xml
│       └── Ranged.xml
│
├── WL/
│   └── 9_WL_Escape_To_B.xml
│
├── KiwiXML          ← Linux binary
├── KiwiXML.exe      ← Windows binary
├── processing.log   ← Output log
└── README.md
```

**Key Rules:**

- Only XML files **one folder deep** (like `KFM/0_KFM_Wolf_New.xml`) are directly processed.
- Files deeper within subfolders (like `KFM/Wolf/Wolf.xml`) are *included* using `#include` instead of being compiled
  directly.
- Output always goes into `./Compiled`.

---

## 🚀 Running the Compiler

### On **Linux**

```bash
./KiwiXML
```

### On **Windows**

```powershell
./KiwiXML.exe
```
Or just open the .exe.

---

By default, the compiler works in the current directory.

You can also specify a directory explicitly:

```bash
./KiwiXML /path/to/source
```

---

## 🧾 Logging

A `processing.log` file is generated each time you run the compiler.  
It includes all processed XMLs, includes, and any missing references.

**Example log:**

```
────────────────────────────────────────────
Starting processing in /home/kvy/KiwiXML/XML Parts
────────────────────────────────────────────
[2025-12-19 22:40:18]  Processed: KFM/0_KFM_Wolf_New.xml
[2025-12-19 22:40:18]  Included:  KFM/Wolf/단타.xml
[2025-12-19 22:40:18]  Included:  KFM/Wolf/연계.xml
[2025-12-19 22:40:18]  Missing include: SF/Earth/Melee.xml
────────────────────────────────────────────
Processing complete. Compiled XMLs saved in ./Compiled
────────────────────────────────────────────
```

---

## 🧩 Example Workflow

1. **Edit modular XMLs**  
   Work directly within structured XMLs instead of editing large files.  
   Example: modify only the files under `KFM/Wolf/` for Wolf‑related changes.

2. **Reference modular files**  
   In your main file (`KFM/0_KFM_Wolf_New.xml`), use:
   ```xml
   <!-- #include file="Wolf\단타.xml" -->
   ```

3. **Run the compiler**
   ```bash
   ./KiwiXML
   ```
   or
   ```powershell
   KiwiXML.exe
   ```

4. **Check `Compiled/` output**  
   You’ll find a fully assembled version of your base XMLs with all includes expanded.

5. **Review the log (`processing.log`)**  
   Confirm everything was included successfully — check for any “Missing include” lines.

---

## 🧩 Features Summary

| Feature             | Description                                        |
|---------------------|----------------------------------------------------|
| Recursive includes  | Automatically combines nested modular XMLs         |
| Cross‑platform      | Works seamlessly on both Windows and Linux         |
| Windows‑style paths | Always interpret `\` in includes correctly         |
| Organized output    | Writes to `./Compiled/`                            |
| Clean logging       | Timestamped log file for every run                 |
| Lightweight         | Single native executable, no installation required |

---

## 🧙 Maintained by

**Kvy**  
Designed for modular XML development workflows.

---
