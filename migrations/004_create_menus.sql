CREATE TABLE menus (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    chef_id UUID NOT NULL REFERENCES chefs(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    price_per_person DECIMAL(10, 2),
    minimum_guests INTEGER DEFAULT 2,
    cuisine_type VARCHAR(100),
    dietary_options TEXT[], -- e.g., ['vegetarian', 'vegan', 'gluten-free']
    duration_hours DECIMAL(4, 2), -- e.g., 2.5 hours
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_menus_chef_id ON menus(chef_id);
CREATE INDEX idx_menus_is_active ON menus(is_active);

