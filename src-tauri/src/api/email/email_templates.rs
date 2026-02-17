/// Template HTML para email de reset de senha
pub fn reset_password_template(token: &str) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Reset Password</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background-color: #f4f4f4; padding: 20px; border-radius: 5px;">
        <h1 style="color: #333; text-align: center;">Forgot Password</h1>
        
        <p>You requested to reset your account password.</p>
        
        <p><strong>To reset your password, follow these steps:</strong></p>
        
        <ol style="line-height: 2; padding-left: 20px;">
            <li>Open the Device App application</li>
            <li>Go to the Forgot Password page</li>
            <li>Enter the reset code below in the "Enter your reset token" field</li>
        </ol>
        
        <div style="background-color: #fff; border: 2px solid #007bff; border-radius: 5px; padding: 15px; margin: 20px 0;">
            <p style="margin: 0 0 10px 0; font-weight: bold; color: #333;">Reset Code:</p>
            <p style="word-break: break-all; color: #007bff; font-family: monospace; font-size: 14px; margin: 0; padding: 10px; background-color: #f8f9fa; border-radius: 3px; user-select: all; -webkit-user-select: all; text-align: center; letter-spacing: 1px;">
                {}
            </p>
            <p style="margin: 10px 0 0 0; font-size: 11px; color: #666; text-align: center;">Click and drag to select the code above, then copy (Ctrl+C / Cmd+C)</p>
        </div>
        
        <p style="margin-top: 30px; font-size: 12px; color: #666;">
            <strong>Important:</strong> This code expires in 20 minutes. If you did not request this reset, please ignore this email.
        </p>
    </div>
</body>
</html>
"#,
        token
    )
}

