# 🔑 NiriKeys

[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)
[![Niri](https://img.shields.io/badge/wm-niri-blue.svg)](https://github.com/YaLTeR/niri)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**NiriKeys** es un gestor interactivo (TUI) avanzado, rápido y seguro para el gestor de ventanas [Niri](https://github.com/YaLTeR/niri) escrito en Rust. Te permite administrar tus atajos de teclado y la apariencia estética de tu entorno directamente desde la terminal, con validación sintáctica activa en tiempo real.

---

## 🎨 Características Principales

*   **Pestaña de Atajos de Teclado**: Añade, edita y elimina tus combinaciones de teclas de forma visual e intuitiva.
*   **Pestaña de Configuración Estética**: Modifica de manera directa el espaciado (gaps), grosor y colores del borde de ventana, grosor y colores del anillo de foco y el radio de redondeo de las esquinas (`geometry-corner-radius`).
*   **Validación de Sintaxis Activa**: Cada cambio es validado automáticamente con `niri validate` en una copia temporal en memoria antes de guardarse físicamente, evitando que tu entorno se rompa.
*   **Importación Inteligente**: Detecta atajos faltantes comparando tu archivo con la plantilla oficial y te permite importarlos uno a uno o todos de un solo golpe.
*   **Respaldos Automáticos**: Crea copias de seguridad de tu archivo de configuración (`config.kdl.bak`) con una sola tecla.
*   **Soporte Multilingüe**: Interfaz completamente adaptada en Español e Inglés con detección automática según el idioma del sistema.

---

## 🚀 Instalación Rápida desde la Terminal

Puedes instalar y usar **NiriKeys** directamente de las siguientes maneras:

### 1. Usando Cargo (Desarrolladores Rust)
Si tienes Rust y Cargo instalados en tu sistema, puedes compilar e instalar la última versión directo desde el repositorio ejecutando:
```bash
cargo install --git https://github.com/MarkOne-dev/NiriKeys.git
```
*(Asegúrate de tener `~/.cargo/bin` en tu variable `$PATH` para poder ejecutarlo con el comando `nirikeys`)*.

---

### 2. Método Manual (Compilar desde el código)
Clona el repositorio, compílalo en modo optimizado y cópialo a tu ruta de ejecutables:
```bash
git clone https://github.com/MarkOne-dev/NiriKeys.git
cd NiriKeys
cargo build --release
cp target/release/nirikeys ~/.local/bin/
```

---

### 3. Descarga Directa del Binario (Para cualquier distribución Linux)
Una vez configurado el lanzamiento en el repositorio, cualquier usuario puede descargar el ejecutable listo para usar y colocarlo en su `$PATH` corriendo una única línea en la terminal:
```bash
curl -L https://github.com/MarkOne-dev/NiriKeys/releases/latest/download/nirikeys -o ~/.local/bin/nirikeys && chmod +x ~/.local/bin/nirikeys
```
*(Nota: Este comando asume que el directorio `~/.local/bin/` existe y está en tu `$PATH`)*.

---

## 🛠️ Controles de la TUI

*   `1`: Cambiar a la pestaña de **Atajos de Teclado**.
*   `2`: Cambiar a la pestaña de **Apariencia Visual**.
*   `↑ / ↓` o `j / k`: Navegar por la lista de atajos o propiedades estéticas.
*   `a`: Añadir un nuevo atajo (solo en pestaña de atajos).
*   `d`: Eliminar el atajo seleccionado (solo en pestaña de atajos).
*   `e` o `Enter`: Editar la propiedad estética seleccionada (solo en pestaña de apariencia).
*   `c`: Mostrar e importar atajos faltantes recomendados desde la plantilla oficial.
*   `b`: Crear una copia de seguridad manual de tu configuración actual.
*   `q` o `Esc`: Salir de la aplicación o cerrar los menús emergentes.

---

## 📄 Licencia

Este proyecto está bajo la Licencia MIT. Consulta el archivo para más detalles.
