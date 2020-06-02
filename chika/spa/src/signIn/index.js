import React from "react";

import SignInForm from "./components/SignInForm";
import "./index.scss";

function SignIn() {
  return (
    <div className="SignIn">
      <h1 className="SignIn__title">Sign In</h1>
      <SignInForm />
    </div>
  );
}

export default SignIn;
