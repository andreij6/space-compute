import React from 'react';
import { Outlet } from 'react-router-dom';
import { Navbar } from '../components/Navbar';
import { Footer } from '../components/Footer';
import { MobileBottomBar } from '../components/MobileBottomBar';

export const RootLayout: React.FC = () => {
  return (
    <div className="app-container">
      <Navbar />
      <main className="content-wrapper">
        <Outlet />
      </main>
      <Footer />
      <MobileBottomBar />
    </div>
  );
};
