CREATE TABLE menu_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    menu_id UUID NOT NULL REFERENCES menus(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    course_type VARCHAR(50), -- e.g., 'appetizer', 'main', 'dessert'
    image_url VARCHAR(500),
    is_featured BOOLEAN DEFAULT false,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_menu_items_menu_id ON menu_items(menu_id);
CREATE INDEX idx_menu_items_is_featured ON menu_items(is_featured);
CREATE INDEX idx_menu_items_display_order ON menu_items(display_order);

