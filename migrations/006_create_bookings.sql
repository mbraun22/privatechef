CREATE TABLE bookings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    chef_id UUID NOT NULL REFERENCES chefs(id) ON DELETE CASCADE,
    customer_id UUID REFERENCES users(id) ON DELETE SET NULL, -- Nullable for guest bookings
    menu_id UUID REFERENCES menus(id) ON DELETE SET NULL,
    customer_name VARCHAR(255) NOT NULL,
    customer_email VARCHAR(255) NOT NULL,
    customer_phone VARCHAR(50),
    event_date DATE NOT NULL,
    event_time TIME NOT NULL,
    duration_hours DECIMAL(4, 2) NOT NULL,
    number_of_guests INTEGER NOT NULL,
    location_address TEXT NOT NULL,
    special_requests TEXT,
    total_price DECIMAL(10, 2) NOT NULL,
    status VARCHAR(50) DEFAULT 'pending', -- pending, confirmed, cancelled, completed
    payment_status VARCHAR(50) DEFAULT 'pending', -- pending, paid, refunded
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_bookings_chef_id ON bookings(chef_id);
CREATE INDEX idx_bookings_customer_id ON bookings(customer_id);
CREATE INDEX idx_bookings_event_date ON bookings(event_date);
CREATE INDEX idx_bookings_status ON bookings(status);
CREATE INDEX idx_bookings_chef_date ON bookings(chef_id, event_date);

