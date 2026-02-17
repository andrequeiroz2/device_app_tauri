import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { motion } from 'framer-motion';
import { AuthLayout } from '@/components/auth/AuthLayout';
import { authApi } from '@/services/authApi';
import { toast } from 'sonner';
import { Loader2, ArrowLeft, CheckCircle } from 'lucide-react';

const ForgotPassword = () => {
  const navigate = useNavigate();
  const [email, setEmail] = useState('');
  const [resetToken, setResetToken] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isSent, setIsSent] = useState(false);
  const [isValidatingToken, setIsValidatingToken] = useState(false);

  const handleTokenSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!resetToken.trim()) {
      toast.error('Enter your reset token');
      return;
    }

    setIsValidatingToken(true);
    try {
      const response = await authApi.validateResetToken(resetToken.trim());
      if (response.success && response.data) {
        navigate(`/reset-password?token=${resetToken.trim()}`);
      } else {
        toast.error(response.message || 'Invalid or expired reset token');
      }
    } catch (error) {
      toast.error('Failed to validate reset token');
    } finally {
      setIsValidatingToken(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!email) {
      toast.error('Enter your email');
      return;
    }

    setIsLoading(true);
    try {
      const response = await authApi.forgotPassword({ email });
      if (response.success) {
        setIsSent(true);
        toast.success('Recovery email sent! Check your inbox.');
      } else {
        toast.error(response.message || 'Error sending email');
      }
    } catch (error) {
      toast.error('Failed to connect to server');
    } finally {
      setIsLoading(false);
    }
  };

  if (isSent) {
    return (
      <AuthLayout title="Email sent" subtitle="Check your inbox">
        <motion.div
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          className="text-center space-y-6"
        >
          <div className="w-16 h-16 bg-accent/10 rounded-full flex items-center justify-center mx-auto">
            <CheckCircle className="w-8 h-8 text-accent" />
          </div>
          
          <div className="space-y-3">
            <p className="text-muted-foreground">
              We sent a recovery link to <strong className="text-foreground">{email}</strong>
            </p>
            
            <div className="bg-muted/50 p-4 rounded-lg border text-left space-y-3">
              <p className="text-sm font-medium text-foreground">
                Next steps:
              </p>
              <ol className="text-sm text-muted-foreground space-y-2 list-decimal list-inside">
                <li>Check your email inbox (and spam folder)</li>
                <li>Copy the reset code from the email</li>
                <li>Return to page "forgotpassword" and enter the code in the "Enter your reset token" field</li>
              </ol>
            </div>
          </div>

          <div className="text-center">
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

  return (
    <AuthLayout
      title="Forgot your password?"
      subtitle="Enter your email to receive a recovery code, or enter your reset token"
    >
      <div className="space-y-6">
        {/* Form para token */}
        <form onSubmit={handleTokenSubmit} className="space-y-4">
          <motion.div
            initial={{ opacity: 0, x: -10 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.1 }}
          >
            <label className="block text-sm font-medium text-foreground mb-2">
              Enter your reset token
            </label>
            <input
              type="text"
              value={resetToken}
              onChange={(e) => setResetToken(e.target.value)}
              className="auth-input font-mono text-sm"
              placeholder="b499a9ba-0c57-4e23-8cca-030cc895aa40"
              disabled={isValidatingToken || isLoading}
            />
          </motion.div>

          <motion.button
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 }}
            type="submit"
            className="auth-button flex items-center justify-center gap-2"
            disabled={isValidatingToken || isLoading || !resetToken.trim()}
          >
            {isValidatingToken ? (
              <>
                <Loader2 className="w-5 h-5 animate-spin" />
                Validating...
              </>
            ) : (
              'Continue with token'
            )}
          </motion.button>
        </form>

        <div className="relative">
          <div className="absolute inset-0 flex items-center">
            <span className="w-full border-t" />
          </div>
          <div className="relative flex justify-center text-xs uppercase">
            <span className="bg-background px-2 text-muted-foreground">Or</span>
          </div>
        </div>

        {/* Form para email */}
        <form onSubmit={handleSubmit} className="space-y-4">
          <motion.div
            initial={{ opacity: 0, x: -10 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.3 }}
          >
            <label className="block text-sm font-medium text-foreground mb-2">Email</label>
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              className="auth-input"
              placeholder="you@example.com"
              disabled={isLoading || isValidatingToken}
            />
          </motion.div>

          <motion.button
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.4 }}
            type="submit"
            className="auth-button flex items-center justify-center gap-2"
            disabled={isLoading || isValidatingToken}
          >
            {isLoading ? (
              <>
                <Loader2 className="w-5 h-5 animate-spin" />
                Sending...
              </>
            ) : (
              'Send recovery code'
            )}
          </motion.button>
        </form>

        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.5 }}
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
      </div>
    </AuthLayout>
  );
};

export default ForgotPassword;
