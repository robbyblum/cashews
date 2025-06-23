import { Link, useMatchRoute } from "@tanstack/react-router";
import ThemeToggle from "./ThemeToggle";
import { useSidebar } from "./ui/sidebar";
import { Button } from "./ui/button";
import { cn } from "@/lib/utils";
import {
  MapIcon,
  MenuIcon,
  PersonStandingIcon,
  VolleyballIcon,
} from "lucide-react";
import { useIsMobile } from "@/hooks/use-mobile";
import { Drawer, DrawerContent, DrawerTrigger } from "./ui/drawer";
import clsx from "clsx";

function SidebarTrigger({
  className,
  onClick,
  ...props
}: React.ComponentProps<typeof Button>) {
  const { toggleSidebar } = useSidebar();

  return (
    <button
      data-sidebar="trigger"
      data-slot="sidebar-trigger"
      variant="ghost"
      className="mx-4"
      onClick={(event) => {
        onClick?.(event);
        toggleSidebar();
      }}
      {...props}
    >
      <MenuIcon className="size-5 mx-0" />
      <span className="sr-only">Toggle Sidebar</span>
    </button>
  );
}

export default function Navbar() {
  const isMobile = useIsMobile();

  const matchRoute = useMatchRoute();
  const pages = [
    { text: "Teams", to: "/teams", icon: VolleyballIcon },
    { text: "Players", to: "/players", icon: PersonStandingIcon },
    { text: "Map", to: "/map", icon: MapIcon },
  ];

  return (
    <header className="bg-gray-100 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-900 flex flex-row">
      <div className="container mx-auto flex flex-row gap-4 items-center pr-2">
        {isMobile && (
          <Drawer direction="left">
            <DrawerTrigger asChild className="cursor-pointer">
              <button className="ml-4">
                <MenuIcon size={24} />
              </button>
            </DrawerTrigger>
            <DrawerContent>
              <DrawerContent>
                <nav className="flex flex-col py-2">
                  {pages.map((page) => {
                    const matches = !!matchRoute({ to: page.to });
                    return (
                      <Link
                        to={page.to}
                        className={clsx(
                          "px-4 py-2 hover:underline hover:bg-muted",
                          matches && "bg-gray-200 dark:bg-gray-700"
                        )}
                      >
                        {page.text}
                      </Link>
                    );
                  })}
                </nav>
              </DrawerContent>
            </DrawerContent>
          </Drawer>
        )}
        <Link to={"/"} className="py-4 font-semibold mr-2">
          🍲 Free Cashews
        </Link>

        {!isMobile &&
          pages.map((page) => {
            return (
              <Link to={page.to} key={page.to} className="py-4">
                {page.text}
              </Link>
            );
          })}

        <div className="flex flex-1">&nbsp;</div>

        <ThemeToggle />
      </div>
    </header>
  );
}
