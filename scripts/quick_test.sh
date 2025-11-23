#!/bin/bash

# Quick test to create menu item
# Usage: ./quick_test.sh <TOKEN> <MENU_ID>

BASE_URL="http://localhost:8080"
API_BASE="$BASE_URL/api"

if [ -z "$1" ] || [ -z "$2" ]; then
    echo "Usage: $0 <TOKEN> <MENU_ID>"
    echo ""
    echo "Example:"
    echo "  $0 'eyJhbGc...' '550e8400-e29b-41d4-a716-446655440000'"
    echo ""
    echo "To get a token, run:"
    echo "  curl -X POST $API_BASE/auth/login \\"
    echo "    -H 'Content-Type: application/json' \\"
    echo "    -d '{\"email\": \"your@email.com\", \"password\": \"password\"}' | jq '.token'"
    echo ""
    exit 1
fi

TOKEN=$1
MENU_ID=$2

echo "Creating menu item..."
echo "URL: $API_BASE/menus/$MENU_ID/items"
echo "Token: $TOKEN"
echo ""

curl -v -X POST "$API_BASE/menus/$MENU_ID/items" \
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
  }' 2>&1 | tee /tmp/menu_item_response.txt

echo ""
echo ""
echo "Response saved to /tmp/menu_item_response.txt"

