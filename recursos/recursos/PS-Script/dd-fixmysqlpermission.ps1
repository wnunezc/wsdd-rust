# Define the path to the directory
$directoryPath = "C:\WSDD-Environment\Docker-Structure\data"

# Get the current ACL (Access Control List) for the directory
$acl = Get-Acl $directoryPath

# Define the permission rule to grant full control to everyone
$permission = "Everyone", "FullControl", "ContainerInherit,ObjectInherit", "None", "Allow"

# Create a new access rule
$accessRule = New-Object System.Security.AccessControl.FileSystemAccessRule $permission

# Add the access rule to the ACL
$acl.AddAccessRule($accessRule)

# Set the ACL for the directory
Set-Acl -Path $directoryPath -AclObject $acl

# Check if the permissions were applied successfully
Get-Acl $directoryPath