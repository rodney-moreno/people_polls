import React, { useState } from 'react';

export interface LoginProps {

}


const Login: React.FC<LoginProps> = () => {
    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");

    return (
        <form onSubmit={e => {
            e.preventDefault();
            fetch("http://localhost:8080/login", {
                method: "POST",
                headers: {"content-type": "application/json"},
                body:  JSON.stringify({email, password}),
                credentials: 'include'
            }).then((res) => {
                if(res.status >= 400 && res.status < 500) {
                    console.log("Username or Password incorrect")
                }
            }) 
            }}>
            <label>Email<input type="text" placeholder="email@email.com" value={email} onChange={e => setEmail(e.target.value)}/></label>
            <label>Password<input type="password" placeholder="password" value={password} onChange={e => setPassword(e.target.value)}/></label>
            <button>Login</button>
            {/* <button>Sign Up</button> */}
        </form>
    )
}

export default Login;