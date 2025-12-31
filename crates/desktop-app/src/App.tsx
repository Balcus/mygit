import { Outlet } from 'react-router-dom';
import Menu from './components/Menu'
import "./App.css";

function App() {
  return (
    <main>
      <Menu />
      <Outlet />
    </main>
  );
}

export default App;
