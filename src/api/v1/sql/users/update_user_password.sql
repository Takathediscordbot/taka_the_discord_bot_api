UPDATE users 
SET password = $1, password_rev = gen_random_uuid() 
WHERE id = $2;