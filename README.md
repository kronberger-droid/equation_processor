# Equation Processor

An intuitive tool for processing and rendering LaTeX-based equations from **Markdown** or **CSV** files into high‑quality PDF/SVG outputs. Offers both a **command‑line interface (CLI)** and a **graphical user interface (GUI)** built with eframe/egui.

---

## Prerequisites

* **Tectonic**: TeX engine used to convert the LaTeX-based equations to PDF:
download for your platform from: [tectonic website](https://tectonic-typesetting.github.io/en-US/) 


* **pdftocairo** (from Poppler): Converts the generated PDFs to SVG:
download for your platform from: [poppler utils](https://poppler.freedesktop.org/)

---

## Supported Input Formats

### 1. Markdown

Place your equations in code‑fenced blocks using `$$` delimiters. Optional flags:

```markdown
%%yes%%
$$
E = mc^2
$$
%%energy_eq%%
```

* **`%%yes%%`** or **`%%no%%`** prefix before `$$` toggles active rendering (default active).
* **`%%custom_name%%`** after closing `$$` sets the output filename (sanitized).

### 2. CSV

A CSV with header row `active,equation,name`:

```csv
active,equation,name
Yes,"\int_0^1 x^2 dx","integral"
No,\frac{a}{b},"fraction_example"
```

* **`Yes`/`No`** in the first column selects active rendering.
* **`equation`** field is raw LaTeX (no surrounding `$$`).
* **`name`** becomes the output filename (duplicates get numbered).

---
