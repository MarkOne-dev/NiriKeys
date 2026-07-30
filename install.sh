#!/usr/bin/env bash

# Colores para salida estética
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0;m' # No Color

echo -e "${BLUE}=== Instalador Automatizado de NiriKeys ===${NC}\n"

# 1. Verificar si Rust/Cargo está instalado
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}[!] Rust no se encuentra instalado en tu sistema.${NC}"
    echo -e "${BLUE}[*] Iniciando instalación automática de Rust a través de rustup...${NC}"
    
    # Descargar e instalar Rust usando el script oficial de rustup de forma no interactiva (-y)
    if curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; then
        echo -e "${GREEN}[✓] Rust se ha instalado correctamente.${NC}"
        # Cargar variables de entorno en el script actual
        source "$HOME/.cargo/env"
    else
        echo -e "${RED}[✗] Error al instalar Rust. Por favor, instálalo de forma manual desde: https://rustup.rs/${NC}"
        exit 1
    fi
else
    echo -e "${GREEN}[✓] Se detectó Rust y Cargo en el sistema.${NC}"
fi

# Asegurar de que cargo esté disponible ahora
if ! command -v cargo &> /dev/null; then
    # Intentar cargar la ruta de cargo explícitamente si se acaba de instalar
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
fi

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}[✗] No se pudo localizar el ejecutable de 'cargo' en el PATH actual. Intenta abrir una nueva terminal y ejecutar la instalación.${NC}"
    exit 1
fi

echo -e "\n${BLUE}[*] Compilando e instalando NiriKeys...${NC}"

# Detectar si estamos ejecutando el script desde el repositorio clonado localmente
if [ -f "Cargo.toml" ]; then
    echo -e "${BLUE}[*] Detectado entorno local de desarrollo. Instalando desde el directorio local...${NC}"
    cargo install --path .
else
    echo -e "${BLUE}[*] Descargando e instalando NiriKeys desde el repositorio remoto oficial...${NC}"
    cargo install --git https://github.com/MarkOne-dev/NiriKeys.git
fi

if [ $? -eq 0 ]; then
    echo -e "\n${GREEN}[✓] ¡NiriKeys se ha instalado con éxito!${NC}"
    echo -e "${YELLOW}[!] Asegúrate de que el directorio '~/.cargo/bin' esté en tu variable PATH.${NC}"
    echo -e "    Puedes comprobarlo escribiendo: ${BLUE}nirikeys${NC}"
else
    echo -e "\n${RED}[✗] Error durante la compilación/instalación de NiriKeys.${NC}"
    exit 1
fi
