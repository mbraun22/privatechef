-- Add quantity/plates_per_item column to menu_items table
-- This allows chefs to specify how many plates of each dish to bring

ALTER TABLE menu_items 
ADD COLUMN IF NOT EXISTS quantity INTEGER DEFAULT 1;

COMMENT ON COLUMN menu_items.quantity IS 'Number of plates/servings for this menu item';

