import { gql, useQuery } from '@apollo/client';
import './App.css';
import { Book, isBookArray } from './types';

const LIST_BOOKS = gql`
  query ListBooks {
    books {
      id
      title
      author
      price
    }
  }
`;

const App = () => {
  const { loading, error, data } = useQuery(LIST_BOOKS);

  if (loading) return <p>Loading...</p>;
  if (error) return <p>Error: {error.message}</p>;

  if (!isBookArray(data.books)) {
    return <p>Error: Invalid data format</p>;
  }
  const books = data.books as Book[];

  return (
    <div className="App">
      <table>
        <thead>
          <tr>
            <th>書籍名</th>
            <th>著者</th>
            <th>価格&nbsp;（円）</th>
          </tr>
        </thead>
        <tbody>
          {books.map((book) => (
            <tr key={book.id}>
              <td>{book.title}</td>
              <td>{book.author}</td>
              <td align="right">{book.price}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

export default App;
