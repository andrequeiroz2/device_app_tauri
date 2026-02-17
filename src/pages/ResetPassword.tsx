import { useState, useEffect } from 'react';
import { useSearchParams, useNavigate, Link } from 'react-router-dom';
import { motion } from 'framer-motion';
import { AuthLayout } from '@/components/auth/AuthLayout';
import { authApi } from '@/services/authApi';
import { toast } from 'sonner';
import { Loader2, Eye, EyeOff, CheckCircle, AlertCircle, ArrowLeft } from 'lucide-react';

interface PasswordValidation {
  minLength: boolean;
  hasLetter: boolean;
  hasNumber: boolean;
  passwordsMatch: boolean;
}

const ResetPassword = () => {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const token = searchParams.get('token');
  
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [isValidating, setIsValidating] = useState(true);
  const [isValid, setIsValid] = useState(false);
  const [isSuccess, setIsSuccess] = useState(false);
  const [userEmail, setUserEmail] = useState('');
  const [manualToken, setManualToken] = useState('');
  const [isProcessingManual, setIsProcessingManual] = useState(false);

  // Validar token ao carregar a página
  useEffect(() => {
    const validateToken = async () => {
      if (!token) {
        toast.error('Invalid reset link. Token is missing.');
        setIsValidating(false);
        return;
      }

      setIsValidating(true);
      try {
        const response = await authApi.validateResetToken(token);
        if (response.success && response.data) {
          setIsValid(true);
          setUserEmail(response.data.email);
        } else {
          toast.error(response.message || 'Invalid or expired reset token');
          setIsValid(false);
        }
      } catch (error) {
        toast.error('Failed to validate reset token');
        setIsValid(false);
      } finally {
        setIsValidating(false);
      }
    };

    validateToken();
  }, [token]);

  // Validação visual de senha
  const validation: PasswordValidation = {
    minLength: password.length >= 6,
    hasLetter: /[a-zA-Z]/.test(password),
    hasNumber: /\d/.test(password),
    passwordsMatch: password === confirmPassword && confirmPassword.length > 0,
  };

  const isPasswordValid = validation.minLength && validation.hasLetter && validation.hasNumber;
  const canSubmit = isPasswordValid && validation.passwordsMatch && !isLoading;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!token) {
      toast.error('Invalid reset link');
      return;
    }

    if (!canSubmit) {
      toast.error('Please fix the errors before submitting');
      return;
    }

    setIsLoading(true);
    try {
      const response = await authApi.resetPassword({
        token,
        password,
        confirm_password: confirmPassword,
      });

      if (response.success) {
        setIsSuccess(true);
        toast.success('Password reset successfully!');
        setTimeout(() => {
          navigate('/login');
        }, 2000);
      } else {
        toast.error(response.message || 'Error resetting password');
      }
    } catch (error) {
      toast.error('Failed to connect to server');
    } finally {
      setIsLoading(false);
    }
  };

  if (isValidating) {
    return (
      <AuthLayout title="Validating" subtitle="Please wait...">
        <div className="flex items-center justify-center py-8">
          <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
        </div>
      </AuthLayout>
    );
  }

  const handleManualTokenSubmit = async () => {
    if (!manualToken.trim()) {
      toast.error('Please enter a reset token');
      return;
    }

    setIsProcessingManual(true);
    // Navegar para a mesma página com o token
    navigate(`/reset-password?token=${manualToken.trim()}`);
  };

  if (!isValid && !token) {
    // Se não há token, mostrar campo para colar o link ou token
    return (
      <AuthLayout title="Reset Password" subtitle="Enter your reset token">
        <motion.div
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          className="space-y-4"
        >
          <div className="bg-muted/50 p-4 rounded-lg border">
            <ol className="text-sm text-muted-foreground space-y-2 list-decimal list-inside">
              <li>Open your email and find the password reset message</li>
              <li>Copy the entire code from the email</li>
              <li>Paste it in the field below</li>
            </ol>
          </div>

          <div>
            <label className="block text-sm font-medium text-foreground mb-2">
              Reset Token
            </label>
            <input
              type="text"
              value={manualToken}
              onChange={(e) => setManualToken(e.target.value)}
              className="auth-input font-mono text-sm"
              placeholder="b499a9ba-0c57-4e23-8cca-030cc895aa40"
              disabled={isProcessingManual}
              autoFocus
            />
          </div>

          <motion.button
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            type="button"
            onClick={handleManualTokenSubmit}
            className="auth-button flex items-center justify-center gap-2"
            disabled={isProcessingManual || !manualToken.trim()}
          >
            {isProcessingManual ? (
              <>
                <Loader2 className="w-5 h-5 animate-spin" />
                Processing...
              </>
            ) : (
              'Continue'
            )}
          </motion.button>

          <div className="text-center mt-6">
            <Link
              to="/login"
              className="text-sm text-muted-foreground hover:text-foreground transition-colors inline-flex items-center gap-1"
            >
              <ArrowLeft className="w-4 h-4" />
              Back to login
            </Link>
          </div>
        </motion.div>
      </AuthLayout>
    );
  }

  if (!isValid) {
    return (
      <AuthLayout title="Invalid Link" subtitle="This reset link is invalid or has expired">
        <motion.div
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          className="text-center"
        >
          <div className="w-16 h-16 bg-destructive/10 rounded-full flex items-center justify-center mx-auto mb-6">
            <AlertCircle className="w-8 h-8 text-destructive" />
          </div>
          <p className="text-muted-foreground mb-6">
            This password reset link is invalid or has expired. Please request a new one.
          </p>
          <Link
            to="/forgot-password"
            className="auth-button inline-flex items-center justify-center gap-2"
          >
            Request New Link
          </Link>
        </motion.div>
      </AuthLayout>
    );
  }

  if (isSuccess) {
    return (
      <AuthLayout title="Success!" subtitle="Your password has been reset">
        <motion.div
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          className="text-center"
        >
          <div className="w-16 h-16 bg-accent/10 rounded-full flex items-center justify-center mx-auto mb-6">
            <CheckCircle className="w-8 h-8 text-accent" />
          </div>
          <p className="text-muted-foreground mb-6">
            Your password has been successfully reset. Redirecting to login...
          </p>
        </motion.div>
      </AuthLayout>
    );
  }

  return (
    <AuthLayout
      title="Reset Password"
      subtitle={`Reset password for ${userEmail}`}
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        <motion.div
          initial={{ opacity: 0, x: -10 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.1 }}
        >
          <label className="block text-sm font-medium text-foreground mb-2">
            New Password
          </label>
          <div className="relative">
            <input
              type={showPassword ? 'text' : 'password'}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="auth-input pr-10"
              placeholder="Enter new password"
              disabled={isLoading}
            />
            <button
              type="button"
              onClick={() => setShowPassword(!showPassword)}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
            >
              {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
            </button>
          </div>
          
          {/* Validação visual de senha */}
          <div className="mt-2 space-y-1 text-xs">
            <div className={`flex items-center gap-2 ${validation.minLength ? 'text-accent' : 'text-muted-foreground'}`}>
              {validation.minLength ? (
                <CheckCircle className="w-3 h-3" />
              ) : (
                <AlertCircle className="w-3 h-3" />
              )}
              At least 6 characters
            </div>
            <div className={`flex items-center gap-2 ${validation.hasLetter ? 'text-accent' : 'text-muted-foreground'}`}>
              {validation.hasLetter ? (
                <CheckCircle className="w-3 h-3" />
              ) : (
                <AlertCircle className="w-3 h-3" />
              )}
              Contains at least one letter
            </div>
            <div className={`flex items-center gap-2 ${validation.hasNumber ? 'text-accent' : 'text-muted-foreground'}`}>
              {validation.hasNumber ? (
                <CheckCircle className="w-3 h-3" />
              ) : (
                <AlertCircle className="w-3 h-3" />
              )}
              Contains at least one number
            </div>
          </div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, x: -10 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.2 }}
        >
          <label className="block text-sm font-medium text-foreground mb-2">
            Confirm Password
          </label>
          <div className="relative">
            <input
              type={showConfirmPassword ? 'text' : 'password'}
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              className={`auth-input pr-10 ${
                confirmPassword.length > 0 && !validation.passwordsMatch
                  ? 'border-destructive focus:border-destructive'
                  : ''
              }`}
              placeholder="Confirm new password"
              disabled={isLoading}
            />
            <button
              type="button"
              onClick={() => setShowConfirmPassword(!showConfirmPassword)}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
            >
              {showConfirmPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
            </button>
          </div>
          {confirmPassword.length > 0 && (
            <div className="mt-2 text-xs flex items-center gap-2">
              {validation.passwordsMatch ? (
                <>
                  <CheckCircle className="w-3 h-3 text-accent" />
                  <span className="text-accent">Passwords match</span>
                </>
              ) : (
                <>
                  <AlertCircle className="w-3 h-3 text-destructive" />
                  <span className="text-destructive">Passwords do not match</span>
                </>
              )}
            </div>
          )}
        </motion.div>

        <motion.button
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
          type="submit"
          className="auth-button flex items-center justify-center gap-2"
          disabled={!canSubmit}
        >
          {isLoading ? (
            <>
              <Loader2 className="w-5 h-5 animate-spin" />
              Resetting...
            </>
          ) : (
            'Reset Password'
          )}
        </motion.button>

        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.4 }}
          className="text-center mt-6"
        >
          <Link
            to="/login"
            className="text-sm text-muted-foreground hover:text-foreground transition-colors inline-flex items-center gap-1"
          >
            <ArrowLeft className="w-4 h-4" />
            Back to login
          </Link>
        </motion.div>
      </form>
    </AuthLayout>
  );
};

export default ResetPassword;

