import { Sidebar, MobileHeader } from "@/components/app/sidebar";
import { GuardrailBanner } from "@/components/app/guardrail-banner";
import { Pomodoro } from "@/components/app/pomodoro";

export default function ShellLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex min-h-dvh">
      <aside className="hidden w-60 shrink-0 border-r bg-card/40 md:block">
        <div className="sticky top-0 h-screen">
          <Sidebar />
        </div>
      </aside>
      <div className="flex min-h-dvh flex-1 flex-col">
        <MobileHeader />
        <main className="mx-auto w-full max-w-5xl flex-1 px-4 py-6 pb-24 md:px-8 md:py-10 md:pb-10">
          <GuardrailBanner />
          {children}
        </main>
      </div>
      <Pomodoro />
    </div>
  );
}
