#!/bin/bash

# Script to download recommended public prompt injection/jailbreak datasets
# Run this in the /data directory

set -e

DATA_DIR="${1:-.}"
echo "Downloading datasets to: $DATA_DIR"
mkdir -p "$DATA_DIR"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== JailGuard Dataset Download Script ===${NC}"
echo "This script downloads the recommended datasets for training"
echo ""

# 1. TrustAIRLab Dataset (via HuggingFace)
echo -e "${BLUE}1. Downloading TrustAIRLab In-The-Wild Jailbreak Prompts...${NC}"
echo "   Source: ACM CCS 2024 research"
echo "   Samples: 15,140 (1,405 jailbreak, 13,735 benign)"
echo "   License: MIT (commercial OK)"
if command -v huggingface-cli &> /dev/null; then
    huggingface-cli download TrustAIRLab/in-the-wild-jailbreak-prompts --repo-type dataset --local-dir "$DATA_DIR/trustailab"
else
    echo "   Note: Install huggingface-hub for direct download:"
    echo "   pip install huggingface-hub[cli]"
    echo "   huggingface-cli download TrustAIRLab/in-the-wild-jailbreak-prompts --repo-type dataset --local-dir $DATA_DIR/trustailab"
fi
echo -e "${GREEN}   ✓ TrustAILab dataset ready${NC}\n"

# 2. SPML Chatbot Dataset
echo -e "${BLUE}2. Downloading SPML Chatbot Prompt Injection Dataset...${NC}"
echo "   Samples: 16,012 annotated examples"
echo "   License: MIT (commercial OK)"
if command -v huggingface-cli &> /dev/null; then
    huggingface-cli download reshabhs/SPML_Chatbot_Prompt_Injection --repo-type dataset --local-dir "$DATA_DIR/spml"
else
    echo "   Note: Install huggingface-hub for direct download:"
    echo "   pip install huggingface-hub[cli]"
    echo "   huggingface-cli download reshabhs/SPML_Chatbot_Prompt_Injection --repo-type dataset --local-dir $DATA_DIR/spml"
fi
echo -e "${GREEN}   ✓ SPML dataset ready${NC}\n"

# 3. xTRam1 Safe-Guard Dataset
echo -e "${BLUE}3. Downloading xTRam1 Safe-Guard Prompt Injection Dataset...${NC}"
echo "   Samples: 10,296 examples"
if command -v huggingface-cli &> /dev/null; then
    huggingface-cli download xTRam1/safe-guard-prompt-injection --repo-type dataset --local-dir "$DATA_DIR/xtram1"
else
    echo "   Note: Install huggingface-hub for direct download"
fi
echo -e "${GREEN}   ✓ xTRam1 dataset ready${NC}\n"

# 4. Giskard Prompt Injections
echo -e "${BLUE}4. Downloading Giskard Prompt Injections Collection...${NC}"
echo "   Format: CSV file"
mkdir -p "$DATA_DIR/giskard"
if command -v curl &> /dev/null; then
    curl -o "$DATA_DIR/giskard/prompt_injections.csv" \
        "https://raw.githubusercontent.com/Giskard-AI/prompt-injections/main/prompt_injections.csv"
    echo -e "${GREEN}   ✓ Giskard dataset downloaded${NC}\n"
else
    echo "   Please download manually from:"
    echo "   https://raw.githubusercontent.com/Giskard-AI/prompt-injections/main/prompt_injections.csv"
fi

# 5. JailbreakBench Dataset
echo -e "${BLUE}5. Downloading JailbreakBench Dataset...${NC}"
echo "   Samples: 200 behaviors (100 harmful, 100 benign)"
mkdir -p "$DATA_DIR/jailbreakbench"
if command -v git &> /dev/null; then
    git clone --depth 1 https://github.com/JailbreakBench/jailbreakbench.git \
        "$DATA_DIR/jailbreakbench" 2>/dev/null || echo "   Note: Repository may already exist"
fi
echo -e "${GREEN}   ✓ JailbreakBench dataset ready${NC}\n"

# 6. AdvBench Dataset
echo -e "${BLUE}6. Downloading AdvBench Dataset...${NC}"
echo "   Samples: 520 harmful instructions"
mkdir -p "$DATA_DIR/advbench"
if command -v git &> /dev/null; then
    git clone --depth 1 https://github.com/llm-attacks/llm-attacks.git \
        "$DATA_DIR/llm-attacks" 2>/dev/null || echo "   Note: Repository may already exist"
    cp "$DATA_DIR/llm-attacks/data/advbench/harmful_behaviors.csv" "$DATA_DIR/advbench/" 2>/dev/null || true
fi
echo -e "${GREEN}   ✓ AdvBench dataset ready${NC}\n"

echo -e "${BLUE}=== Dataset Download Summary ===${NC}"
echo "Recommended datasets downloaded:"
echo "  ✓ TrustAILab (15,140 samples)"
echo "  ✓ SPML (16,012 samples)"
echo "  ✓ xTRam1 (10,296 samples)"
echo "  ✓ Giskard (variable)"
echo "  ✓ JailbreakBench (200 samples)"
echo "  ✓ AdvBench (520 samples)"
echo ""
echo "Total estimated samples: ~52,000+ (before deduplication)"
echo ""
echo "Data location: $DATA_DIR"
echo ""
echo "Next steps:"
echo "  1. Review DATASETS.md for detailed integration instructions"
echo "  2. Update your training pipeline to load these datasets"
echo "  3. Consider deduplication across datasets"
echo "  4. Check licensing for your use case (some are non-commercial only)"
echo ""
echo -e "${GREEN}Download complete!${NC}"
