import { QueryClientProvider } from "@tanstack/react-query";
import { queryClient } from "@/lib/query";
import { Dashboard } from "./dashboard";

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <Dashboard />
    </QueryClientProvider>
  );
}
