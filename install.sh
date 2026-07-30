#!/usr/bin/env bash

# Colores para salida estética
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0;m' # No Color

echo -e "${BLUE}=== Instalador Automatizado de NiriKeys ===${NC}\n"

# 1. Verificar si Rust/Cargo está instalado en el sistema
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}[!] Rust (motor de compilación) no se encuentra instalado en tu sistema.${NC}"
    
    # Detectar el gestor de paquetes de la distro para instalar Rust/Cargo
    INSTALL_RUST_CMD=""
    PM_NAME=""
    
    if command -v pacman &> /dev/null; then
        PM_NAME="Arch Linux (pacman)"
        INSTALL_RUST_CMD="sudo pacman -S --needed rust cargo"
    elif command -v dnf &> /dev/null; then
        PM_NAME="Fedora (dnf)"
        INSTALL_RUST_CMD="sudo dnf install -y rust cargo"
    elif command -v zypper &> /dev/null; then
        PM_NAME="openSUSE (zypper)"
        INSTALL_RUST_CMD="sudo zypper install -y rust cargo"
    elif command -v apt-get &> /dev/null; then
        PM_NAME="Debian/Ubuntu (apt-get)"
        INSTALL_RUST_CMD="sudo apt-get update && sudo apt-get install -y rustc cargo"
    fi
    
    if [ -n "$INSTALL_RUST_CMD" ]; then
        echo -e "${BLUE}[*] Gestor de paquetes detectado: $PM_NAME${NC}"
        read -p "¿Deseas instalar Rust y Cargo usando '$INSTALL_RUST_CMD'? [S/n]: " opt
        opt=$(echo "$opt" | tr '[:upper:]' '[:lower:]')
        if [[ -z "$opt" || "$opt" == "s" || "$opt" == "si" || "$opt" == "y" || "$opt" == "yes" ]]; then
            echo -e "${BLUE}[*] Ejecutando instalación de Rust...${NC}"
            if eval "$INSTALL_RUST_CMD"; then
                echo -e "${GREEN}[✓] Rust y Cargo se han instalado correctamente desde los repositorios de tu sistema.${NC}"
            else
                echo -e "${RED}[✗] Falló la instalación mediante el gestor de paquetes.${NC}"
                # Fallback al script oficial de rustup
                echo -e "${BLUE}[*] Intentando instalación alternativa mediante el script oficial de rustup...${NC}"
                if curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; then
                    echo -e "${GREEN}[✓] Rust se ha instalado correctamente mediante rustup.${NC}"
                    source "$HOME/.cargo/env"
                else
                    echo -e "${RED}[✗] Error al instalar Rust de forma alternativa. Por favor, instálalo manualmente desde: https://rustup.rs/${NC}"
                    exit 1
                fi
            fi
        else
            echo -e "${RED}[✗] Instalación de Rust omitida. NiriKeys requiere Rust para compilarse.${NC}"
            exit 1
        fi
    else
        # Fallback directo si no se detecta gestor de paquetes soportado
        echo -e "${BLUE}[*] Iniciando instalación automática de Rust a través de rustup (oficial)...${NC}"
        if curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; then
            echo -e "${GREEN}[✓] Rust se ha instalado correctamente.${NC}"
            source "$HOME/.cargo/env"
        else
            echo -e "${RED}[✗] Error al instalar Rust. Por favor, instálalo de forma manual desde: https://rustup.rs/${NC}"
            exit 1
        fi
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
