import { Link } from 'react-router-dom';
import { motion } from 'framer-motion';
import { LogIn, UserPlus, Lock } from 'lucide-react';

const Index = () => {
  return (
    <div className="min-h-screen flex items-center justify-center bg-secondary/30 px-4">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="text-center"
      >
        <motion.div
          initial={{ scale: 0.9, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          transition={{ delay: 0.1 }}
          className="w-16 h-16 bg-primary rounded-2xl flex items-center justify-center mx-auto mb-8"
        >
          <Lock className="w-8 h-8 text-primary-foreground" />
        </motion.div>
        
        <h1 className="text-4xl font-semibold text-foreground mb-3">
          Sistema de Autenticação
        </h1>
        <p className="text-lg text-muted-foreground mb-10 max-w-md">
          Interface moderna e minimalista para login, registro e recuperação de senha.
        </p>

        <div className="flex flex-col sm:flex-row gap-4 justify-center">
          <motion.div
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.3 }}
          >
            <Link
              to="/login"
              className="auth-button inline-flex items-center justify-center gap-2 px-8"
            >
              <LogIn className="w-5 h-5" />
              Entrar
            </Link>
          </motion.div>
          
          <motion.div
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.4 }}
          >
            <Link
              to="/register"
              className="inline-flex items-center justify-center gap-2 px-8 h-12 rounded-xl border-2 border-primary text-primary font-medium hover:bg-primary hover:text-primary-foreground transition-all duration-200"
            >
              <UserPlus className="w-5 h-5" />
              Criar conta
            </Link>
          </motion.div>
        </div>
      </motion.div>
    </div>
  );
};

export default Index;
