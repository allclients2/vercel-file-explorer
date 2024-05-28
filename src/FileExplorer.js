import React, { useState, useEffect } from 'react';
import axios from 'axios';

const FileExplorer = () => {
  const [files, setFiles] = useState([]);
  const [file, setFile] = useState(null);
  const [authenticated, setAuthenticated] = useState(false);
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');

  useEffect(() => {
    fetchFiles();
  }, []);

  const fetchFiles = () => {
    axios.get('/api/files')
      .then(response => setFiles(response.data))
      .catch(error => console.error('Error fetching files:', error));
  };

  // react is fire lol

  const handleUpload = () => {
    if (file) {
      const formData = new FormData();
      formData.append('file', file);
      axios.post('/api/upload', formData, {
        headers: {
          'Content-Type': 'multipart/form-data',
        },
        auth: {
          username,
          password,
        },
      })
        .then(() => {
          setFile(null);
          fetchFiles();
        })
        .catch(error => {
          alert('Authentication failed or upload error');
          console.error('Upload error:', error);
        });
    }
  };

  return (
    <div>
      <h1>File Explorer Thing</h1>
      <ul>
        {files.map(f => (
          <li key={f.name}>
            <a href={`/api/files/${f.name}`} download>{f.name}</a>
          </li>
        ))}
      </ul>
      {authenticated && (
        <div>
          <input type="file" onChange={e => setFile(e.target.files[0])} />
          <button onClick={handleUpload}>Upload</button>
        </div>
      )}
      {!authenticated && (
        <div>
          <input
            type="text"
            placeholder="Username"
            value={username}
            onChange={e => setUsername(e.target.value)}
          />
          <input
            type="password"
            placeholder="Password"
            value={password}
            onChange={e => setPassword(e.target.value)}
          />
          <button onClick={() => setAuthenticated(true)}>Authenticate</button>
        </div>
      )}
    </div>
  );
};

export default FileExplorer;
