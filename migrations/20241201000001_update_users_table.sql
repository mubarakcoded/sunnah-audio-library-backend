-- Update users table to use UUID and ensure compatibility
CREATE TABLE IF NOT EXISTS `tbl_users` (
  `id` int NOT NULL AUTO_INCREMENT,
  `user_id` varchar(36) NOT NULL UNIQUE,
  `username` varchar(255) NOT NULL UNIQUE,
  `password` varchar(255) NOT NULL,
  `role` enum('Admin','Manager','Viewer','ServiceAdmin','ServiceManager','ServiceViewer') NOT NULL DEFAULT 'Viewer',
  `created_at` timestamp DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `idx_user_id` (`user_id`),
  KEY `idx_username` (`username`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Update access table to ensure proper foreign key relationships
ALTER TABLE `tbl_access` 
ADD COLUMN IF NOT EXISTS `id` int NOT NULL AUTO_INCREMENT PRIMARY KEY FIRST,
MODIFY COLUMN `user_id` varchar(36) NOT NULL,
MODIFY COLUMN `created_by` varchar(36) NOT NUL L,
ADD INDEX IF NOT EXISTS `idx_user_scholar` (`user_id`, `scholar_id`),
ADD INDEX IF NOT EXISTS `idx_scholar_id` (`scholar_id`);

-- Update files table to support uploads
ALTER TABLE `tbl_files` 
ADD COLUMN IF NOT EXISTS `uploaded_by` varchar(36) NULL,
ADD COLUMN IF NOT EXISTS `file_size` bigint DEFAULT 0,
ADD COLUMN IF NOT EXISTS `content_type` varchar(255) DEFAULT 'application/octet-stream',
ADD INDEX IF NOT EXISTS `idx_uploaded_by` (`uploaded_by`);