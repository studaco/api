ALTER TABLE account DROP COLUMN pssword_hash;
ALTER TABLE account ADD COLUMN password_hash TEXT NOT NULL;