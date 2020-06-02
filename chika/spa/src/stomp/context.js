import React from 'react';

const StompContext = React.createContext(null);

export const withStomp = (Component) => (props) => (
  <StompContext.Consumer>
    {(stomp) => <Component {...props} stomp={stomp} />}
  </StompContext.Consumer>
);

export default StompContext;
