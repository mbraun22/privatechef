-- Add role column to users table
-- Roles: admin, mod, chef, diner
-- Default role is 'diner' for new users

ALTER TABLE users 
ADD COLUMN role VARCHAR(20) NOT NULL DEFAULT 'diner';

-- Add constraint to ensure only valid roles are allowed
ALTER TABLE users 
ADD CONSTRAINT check_role 
CHECK (role IN ('admin', 'mod', 'chef', 'diner'));

-- Create index for role-based queries
CREATE INDEX idx_users_role ON users(role);

-- Update existing users to have 'diner' role if they don't have one
UPDATE users SET role = 'diner' WHERE role IS NULL;

