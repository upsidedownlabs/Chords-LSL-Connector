"use client";

import React from 'react';
import dynamic from 'next/dynamic';
const ClientOnlyPage = dynamic(() => import('./main'), { ssr: false });

const Page = () => {
  return (
    <div>
      <ClientOnlyPage />
    </div>
  );
};

export default Page;