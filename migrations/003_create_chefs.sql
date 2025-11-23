CREATE TABLE chefs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    business_name VARCHAR(255),
    chef_name VARCHAR(255) NOT NULL,
    bio TEXT,
    cuisine_types TEXT[], -- Array of cuisine types
    location VARCHAR(255),
    phone VARCHAR(50),
    email VARCHAR(255),
    website VARCHAR(255),
    profile_image_url VARCHAR(500),
    cover_image_url VARCHAR(500),
    hourly_rate DECIMAL(10, 2),
    minimum_hours INTEGER DEFAULT 2,
    travel_radius INTEGER, -- in miles
    is_active BOOLEAN DEFAULT true,
    slug VARCHAR(255) UNIQUE, -- For SEO-friendly URLs
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_chefs_user_id ON chefs(user_id);
CREATE INDEX idx_chefs_slug ON chefs(slug);
CREATE INDEX idx_chefs_location ON chefs(location);
CREATE INDEX idx_chefs_is_active ON chefs(is_active);

