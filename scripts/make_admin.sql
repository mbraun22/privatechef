-- Script to make a user an admin
-- Usage: Update the email below and run: psql $DATABASE_URL -f scripts/make_admin.sql

-- Update user role to admin (replace 'mattbraun@example.com' with actual email)
UPDATE users 
SET role = 'admin' 
WHERE email LIKE '%mattbraun%' OR email LIKE '%matt%braun%';

-- Verify the update
SELECT id, email, role FROM users WHERE role = 'admin';

