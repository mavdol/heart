import React from "react";
import ReactDOM from "react-dom/client";
import RootLayout from "@/layout";
import Chat from "@/chat";
import CustomProviders from "@/providers";

import "virtual:uno.css";
import "@/styles/globals.scss";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <div className="w-full min-h-screen">
      <CustomProviders>
        <RootLayout>
          <Chat />
        </RootLayout>
      </CustomProviders>
    </div>
  </React.StrictMode>
);
