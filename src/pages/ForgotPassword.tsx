import { useState } from 'react';
import { Link } from 'react-router-dom';
import { motion } from 'framer-motion';
import { AuthLayout } from '@/components/auth/AuthLayout';
import { authApi } from '@/services/authApi';
import { toast } from 'sonner';
import { Loader2, ArrowLeft, CheckCircle } from 'lucide-react';

const ForgotPassword = () => {
  const [email, setEmail] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isSent, setIsSent] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!email) {
      toast.error('Enter your email');
      return;
    }

    // Backend does not expose forgot-password yet; avoid calling to prevent errors.
    toast.error('Password recovery is not available yet.');
    return;

    setIsLoading(true);
    try {
      const response = await authApi.forgotPassword(email);
      if (response.success) {
        setIsSent(true);
        toast.success('Recovery email sent!');
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
          className="text-center"
        >
          <div className="w-16 h-16 bg-accent/10 rounded-full flex items-center justify-center mx-auto mb-6">
            <CheckCircle className="w-8 h-8 text-accent" />
          </div>
          <p className="text-muted-foreground mb-6">
            We sent a recovery link to <strong className="text-foreground">{email}</strong>
          </p>
          <Link
            to="/login"
            className="auth-button inline-flex items-center justify-center gap-2"
          >
            <ArrowLeft className="w-4 h-4" />
            Back to login
          </Link>
        </motion.div>
      </AuthLayout>
    );
  }

  return (
    <AuthLayout
      title="Forgot your password?"
      subtitle="Enter your email to receive a recovery link"
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        <motion.div
          initial={{ opacity: 0, x: -10 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.2 }}
        >
          <label className="block text-sm font-medium text-foreground mb-2">Email</label>
          <input
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            className="auth-input"
            placeholder="you@example.com"
            disabled={isLoading}
          />
        </motion.div>

        <motion.button
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
          type="submit"
          className="auth-button flex items-center justify-center gap-2"
          disabled={isLoading}
        >
          {isLoading ? (
            <>
              <Loader2 className="w-5 h-5 animate-spin" />
              Sending...
            </>
          ) : (
            'Send recovery link'
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

export default ForgotPassword;
