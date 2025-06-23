import type { ReactNode } from "react";
import {
  Outlet,
  createRootRoute,
  HeadContent,
  Scripts,
  useLocation,
} from "@tanstack/react-router";

import appCss from "@/styles/app.css?url";

import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import { ThemeProvider } from "next-themes";
import clsx from "clsx";
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";
import AppSidebar from "@/components/AppSidebar";

export const Route = createRootRoute({
  head: () => ({
    meta: [
      {
        charSet: "utf-8",
      },
      {
        name: "viewport",
        content: "width=device-width, initial-scale=1",
      },
      {
        title: "Free Cashews",
      },
    ],
    links: [
      {
        rel: "stylesheet",
        href: appCss,
      },
    ],
  }),
  component: RootComponent,
});

function RootComponent() {
  return (
    <RootDocument>
      <Outlet />
    </RootDocument>
  );
}

function RootDocument({ children }: Readonly<{ children: ReactNode }>) {
  const location = useLocation();

  const isMap = location.href === "/map";

  return (
    <html suppressHydrationWarning>
      <head>
        <HeadContent />
      </head>
      <body className={clsx("flex flex-row w-screen", isMap && "h-screen")}>
        <ThemeProvider attribute="class">
          <div className="flex-1 flex flex-col w-full">
            <Navbar />

            <main className="flex-1 max-w-full overflow-x-auto">
              {children}
            </main>
            {!isMap && <Footer />}
          </div>
          <Scripts />
        </ThemeProvider>
      </body>
    </html>
  );
}
