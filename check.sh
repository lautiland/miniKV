#!/bin/bash

# Colores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

LOG_FILE="check.log"
FAILED=0

# Limpiar log anterior al inicio
rm -f "$LOG_FILE"

echo -e "${YELLOW}=== MiniKV Check ===${NC}"
echo ""

# 1. Format check
echo -n "Checking format... "
FMT_OUTPUT=$(cargo fmt --check 2>&1)
if [ $? -eq 0 ]; then
    echo -e "${GREEN}OK${NC}"
else
    echo -e "${RED}FAILED${NC}"
    FAILED=1
    echo "=== cargo fmt --check ===" >> "$LOG_FILE"
    echo "$FMT_OUTPUT" >> "$LOG_FILE"
    echo "" >> "$LOG_FILE"
fi

# 2. Clippy pedantic
echo -n "Checking clippy pedantic... "
CLIPPY_OUTPUT=$(cargo clippy --tests -- -D warnings -D clippy::pedantic 2>&1)
if [ $? -eq 0 ]; then
    echo -e "${GREEN}OK${NC}"
else
    echo -e "${RED}FAILED${NC}"
    FAILED=1
    echo "=== cargo clippy ===" >> "$LOG_FILE"
    echo "$CLIPPY_OUTPUT" >> "$LOG_FILE"
    echo "" >> "$LOG_FILE"
fi

# 3. Tests
echo -n "Running tests... "
TEST_OUTPUT=$(cargo test 2>&1)
if [ $? -eq 0 ]; then
    echo -e "${GREEN}OK${NC}"
else
    echo -e "${RED}FAILED${NC}"
    FAILED=1
    echo "=== cargo test ===" >> "$LOG_FILE"
    echo "$TEST_OUTPUT" >> "$LOG_FILE"
    echo "" >> "$LOG_FILE"
fi

# Resultado final
echo ""
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}=== ALL CHECKS PASSED ===${NC}"
else
    echo -e "${RED}=== SOME CHECKS FAILED ===${NC}"
    echo -e "Ver detalles en: ${YELLOW}${LOG_FILE}${NC}"
fi

exit $FAILED
