UPDATE users 
SET 
name = $1, 
email = $2, 
role = $3, 
verified = $4, 
updated_at = $5 
WHERE id = $6::uuid
RETURNING *;
