import { Navigate } from "react-router-dom";
import { useAuth } from "@/context/AuthContext";

interface ProtectedRouteProps {
  children: React.ReactElement;
}

export const ProtectedRoute = ({ children }: ProtectedRouteProps) => {
  const { token, isLoading } = useAuth();

  if (isLoading) {
    return null;
  }

  if (!token) {
    return <Navigate to="/login" replace />;
  }

  return children;
};


