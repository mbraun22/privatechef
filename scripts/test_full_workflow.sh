#!/bin/bash

# Full workflow test: User registration -> Authentication -> Chef profile -> Menu creation -> Menu item creation
# This script will help identify where the menu item creation is failing

set -e

BASE_URL="http://localhost:8080"
API_BASE="$BASE_URL/api"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Full Workflow Test ===${NC}\n"

# Step 1: Check if server is running
echo -e "${YELLOW}[1] Checking if server is running...${NC}"
if ! curl -s "$BASE_URL/" > /dev/null 2>&1; then
    echo -e "${RED}✗ Server is not running on $BASE_URL${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Server is running${NC}\n"

# Step 2: Create or login user
echo -e "${YELLOW}[2] User Setup${NC}"
echo "Note: For testing, you need to:"
echo "  1. Register a user (if not already done)"
echo "  2. Have credentials ready for login"
echo ""

# Example: Register a user (you would need to implement this endpoint)
echo "Example registration:"
echo "curl -X POST \"$API_BASE/auth/register\" \\"
echo "  -H \"Content-Type: application/json\" \\"
echo "  -d '{\"email\": \"chef@example.com\", \"password\": \"password123\"}'"
echo ""

# Step 3: Login and get token
echo -e "${YELLOW}[3] Getting Authentication Token${NC}"
echo "Example login:"
echo "curl -X POST \"$API_BASE/auth/login\" \\"
echo "  -H \"Content-Type: application/json\" \\"
echo "  -d '{\"email\": \"chef@example.com\", \"password\": \"password123\"}' | jq '.token'"
echo ""
echo "Save the returned token in TOKEN variable:"
echo "TOKEN=<your_token_here>"
echo ""

# Interactive token input
read -p "Enter your JWT token (or press Enter to skip): " TOKEN

if [ -z "$TOKEN" ]; then
    echo -e "${RED}✗ No token provided. Cannot continue.${NC}"
    echo "Please:"
    echo "1. Register a user with the backend"
    echo "2. Login to get a JWT token"
    echo "3. Re-run this script with your token"
    exit 1
fi

echo -e "${GREEN}✓ Token provided${NC}\n"

# Step 4: Get chef profile (verify user is a chef)
echo -e "${YELLOW}[4] Checking Chef Profile${NC}"
CHEF_RESPONSE=$(curl -s -X GET "$API_BASE/chefs/profile" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json")

echo "Response: $CHEF_RESPONSE"
echo ""

# Check if error
if echo "$CHEF_RESPONSE" | grep -q "error\|unauthorized"; then
    echo -e "${YELLOW}Note: User is not a chef yet. Creating chef profile...${NC}"
    
    # Step 5: Create chef profile if needed
    echo -e "${YELLOW}[5] Creating Chef Profile${NC}"
    CHEF_CREATE_RESPONSE=$(curl -s -X POST "$API_BASE/chefs" \
      -H "Authorization: Bearer $TOKEN" \
      -H "Content-Type: application/json" \
      -d '{
        "name": "Test Chef",
        "specialty": "French Cuisine",
        "bio": "Test chef for menu item creation",
        "experience_years": 5,
        "location": "San Francisco, CA"
      }')
    
    echo "Chef creation response: $CHEF_CREATE_RESPONSE"
    
    if echo "$CHEF_CREATE_RESPONSE" | grep -q "error"; then
        echo -e "${RED}✗ Failed to create chef profile${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ Chef profile created${NC}\n"
else
    echo -e "${GREEN}✓ User is a chef${NC}\n"
    CHEF_ID=$(echo "$CHEF_RESPONSE" | jq -r '.id' 2>/dev/null || echo "")
    echo "Chef ID: $CHEF_ID\n"
fi

# Step 6: Get menus
echo -e "${YELLOW}[6] Fetching Menus${NC}"
MENUS_RESPONSE=$(curl -s -X GET "$API_BASE/menus" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json")

echo "Menus response: $MENUS_RESPONSE"
echo ""

# Parse menu ID
MENU_ID=$(echo "$MENUS_RESPONSE" | jq -r '.[0].id' 2>/dev/null || echo "")

if [ -z "$MENU_ID" ] || [ "$MENU_ID" == "null" ]; then
    echo -e "${YELLOW}No menus found. Creating a new menu...${NC}"
    
    # Step 7: Create menu if needed
    echo -e "${YELLOW}[7] Creating Menu${NC}"
    MENU_CREATE_RESPONSE=$(curl -s -X POST "$API_BASE/menus" \
      -H "Authorization: Bearer $TOKEN" \
      -H "Content-Type: application/json" \
      -d '{
        "name": "Test Menu",
        "description": "A test menu for menu item creation",
        "cuisine_type": "French"
      }')
    
    echo "Menu creation response: $MENU_CREATE_RESPONSE"
    
    MENU_ID=$(echo "$MENU_CREATE_RESPONSE" | jq -r '.id' 2>/dev/null || echo "")
    
    if [ -z "$MENU_ID" ] || [ "$MENU_ID" == "null" ]; then
        echo -e "${RED}✗ Failed to create menu${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✓ Menu created${NC}\n"
else
    echo -e "${GREEN}✓ Menu found${NC}\n"
fi

echo "Using Menu ID: $MENU_ID"
echo ""

# Step 8: Create menu item (THE MAIN TEST)
echo -e "${YELLOW}[8] Creating Menu Item (Main Test)${NC}"
echo "Endpoint: POST $API_BASE/menus/$MENU_ID/items"
echo "Token: Bearer $TOKEN"
echo ""

MENU_ITEM_RESPONSE=$(curl -v -X POST "$API_BASE/menus/$MENU_ID/items" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Pan-Seared Salmon",
    "description": "Fresh Atlantic salmon with lemon butter",
    "course_type": "main",
    "image_url": "https://example.com/salmon.jpg",
    "is_featured": true,
    "display_order": 1,
    "quantity": 4
  }' 2>&1)

echo ""
echo "Full response:"
echo "$MENU_ITEM_RESPONSE"
echo ""

# Parse response
ITEM_ID=$(echo "$MENU_ITEM_RESPONSE" | grep -A 100 "{" | jq -r '.id' 2>/dev/null || echo "")

if [ -z "$ITEM_ID" ] || [ "$ITEM_ID" == "null" ]; then
    echo -e "${RED}✗ Failed to create menu item${NC}"
    echo ""
    echo "Debugging information:"
    echo "- Menu ID: $MENU_ID"
    echo "- Token: $TOKEN"
    echo ""
    echo "Check the server logs for more details about the error"
    exit 1
fi

echo -e "${GREEN}✓ Menu item created successfully!${NC}"
echo "Item ID: $ITEM_ID"
echo ""

# Step 9: Verify menu item was created
echo -e "${YELLOW}[9] Verifying Menu Item Creation${NC}"
GET_ITEMS_RESPONSE=$(curl -s -X GET "$API_BASE/menus/$MENU_ID/items" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json")

echo "Menu items: $GET_ITEMS_RESPONSE"
echo ""

if echo "$GET_ITEMS_RESPONSE" | grep -q "$ITEM_ID"; then
    echo -e "${GREEN}✓ Menu item verified in database${NC}\n"
else
    echo -e "${YELLOW}⚠ Could not verify menu item in list${NC}\n"
fi

echo -e "${GREEN}=== Workflow Test Complete ===${NC}"

