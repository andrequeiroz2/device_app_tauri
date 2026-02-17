import { useEffect } from "react";
import { ThemeProvider } from "@/components/theme-provider";
import { Toaster } from "@/components/ui/toaster";
import { Toaster as Sonner } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Routes, Route, Link, useNavigate } from "react-router-dom";
import { AuthProvider, useAuth } from "@/context/AuthContext";
import { ProtectedRoute } from "@/components/auth/ProtectedRoute";
import { ThemeToggle } from "@/components/ThemeToggle";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { Button } from "@/components/ui/button";
import { User, LogOut, Lock } from "lucide-react";
import Index from "./pages/Index";
import Login from "./pages/Login";
import Register from "./pages/Register";
import ForgotPassword from "./pages/ForgotPassword";
import ResetPassword from "./pages/ResetPassword";
import ChangePassword from "./pages/ChangePassword";
import NotFound from "./pages/NotFound";
import Locations from "./pages/Locations";
import LocationsList from "./pages/LocationsList";
import LocationDetail from "./pages/LocationDetail";
import LocationEdit from "./pages/LocationEdit";
import Home from "./pages/Home";
import MqttBrokerCreate from "./pages/MqttBrokerCreate";
import MqttBrokersList from "./pages/MqttBrokersList";
import MqttBrokerDetail from "./pages/MqttBrokerDetail";

const queryClient = new QueryClient();

const UserMenu = () => {
  const { token, logout } = useAuth();
  if (!token) return null;

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" aria-label="User menu">
          <User className="w-5 h-5" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem asChild>
          <Link to="/change-password" className="w-full">
            <Lock className="w-4 h-4 mr-2" />
            Change Password
          </Link>
        </DropdownMenuItem>
        <DropdownMenuItem
          onSelect={(e) => {
            e.preventDefault();
            logout();
          }}
        >
          <LogOut className="w-4 h-4 mr-2" />
          Logout
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

const NavBar = () => {
  const { token } = useAuth();
  if (!token) return null;

  return (
    <header className="border-b bg-background/80 backdrop-blur">
      <div className="max-w-6xl mx-auto flex items-center justify-between px-4 py-3">
        <div className="flex items-center gap-3">
          <nav className="flex items-center gap-3 text-sm text-muted-foreground">
            <Link to="/" className="hover:text-foreground transition-colors">
              Home
            </Link>
            <DropdownMenu>
              <DropdownMenuTrigger className="hover:text-foreground transition-colors">
                Location
              </DropdownMenuTrigger>
              <DropdownMenuContent align="start">
                <DropdownMenuItem asChild>
                  <Link to="/locations/list" className="w-full">
                    List
                  </Link>
                </DropdownMenuItem>
                <DropdownMenuItem asChild>
                  <Link to="/locations/create" className="w-full">
                    Create
                  </Link>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
            <DropdownMenu>
              <DropdownMenuTrigger className="hover:text-foreground transition-colors">
                Broker
              </DropdownMenuTrigger>
              <DropdownMenuContent align="start">
                <DropdownMenuItem asChild>
                  <Link to="/mqtt-brokers/list" className="w-full">
                    List
                  </Link>
                </DropdownMenuItem>
                <DropdownMenuItem asChild>
                  <Link to="/mqtt-brokers/create" className="w-full">
                    Create
                  </Link>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </nav>
        </div>
        <div className="flex items-center gap-2">
          <ThemeToggle />
          <UserMenu />
        </div>
      </div>
    </header>
  );
};

const DeepLinkHandler = () => {
  const navigate = useNavigate();

  useEffect(() => {
    // No Tauri desktop, quando o link tauri://reset-password?token=... é clicado,
    // o sistema operacional tenta abrir a aplicação. Se já estiver aberta,
    // precisamos capturar isso de outra forma.
    
    // Por enquanto, vamos verificar se há um token na URL quando a página carrega
    // Isso funciona quando o usuário copia e cola o link manualmente
    const checkUrlForToken = () => {
      const urlParams = new URLSearchParams(window.location.search);
      const token = urlParams.get('token');
      if (token && window.location.pathname !== '/reset-password') {
        navigate(`/reset-password?token=${token}`);
      }
    };

    checkUrlForToken();
  }, [navigate]);

  return null;
};

const App = () => (
  <QueryClientProvider client={queryClient}>
    <AuthProvider>
      <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
    <TooltipProvider>
          <Toaster />
      <Sonner position="top-center" />
      <BrowserRouter>
            <DeepLinkHandler />
            <div className="min-h-screen bg-background text-foreground">
              <NavBar />
              <div className="max-w-6xl mx-auto px-4 py-4">
        <Routes>
                <Route
                  path="/"
                  element={
                    <ProtectedRoute>
                      <Home />
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/welcome"
                  element={
                    <ProtectedRoute>
                      <Index />
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/locations/create"
                  element={
                    <ProtectedRoute>
                      <Locations />
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/locations/list"
                  element={
                    <ProtectedRoute>
                      <LocationsList />
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/locations/:uuid"
                  element={
                    <ProtectedRoute>
                      <LocationDetail />
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/locations/:uuid/edit"
                  element={
                    <ProtectedRoute>
                      <LocationEdit />
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/mqtt-brokers/create"
                  element={
                    <ProtectedRoute>
                      <MqttBrokerCreate />
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/mqtt-brokers/list"
                  element={
                    <ProtectedRoute>
                      <MqttBrokersList />
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/mqtt-brokers/:uuid"
                  element={
                    <ProtectedRoute>
                      <MqttBrokerDetail />
                    </ProtectedRoute>
                  }
                />
          <Route path="/login" element={<Login />} />
          <Route path="/register" element={<Register />} />
          <Route path="/forgot-password" element={<ForgotPassword />} />
          <Route path="/reset-password" element={<ResetPassword />} />
          <Route path="/change-password" element={<ProtectedRoute><ChangePassword /></ProtectedRoute>} />
          <Route path="*" element={<NotFound />} />
        </Routes>
              </div>
            </div>
      </BrowserRouter>
    </TooltipProvider>
      </ThemeProvider>
    </AuthProvider>
  </QueryClientProvider>
);

export default App;
