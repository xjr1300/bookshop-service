export type Book = {
  id: number;
  title: string;
  author: string;
  price: number;
};

export const isBook = (obj: unknown): obj is Book => {
  const book = obj as Book;
  return (
    typeof book.id === 'number' &&
    typeof book.title === 'string' &&
    typeof book.author === 'string' &&
    typeof book.price === 'number'
  );
};

export const isBookArray = (obj: unknown): obj is Book[] => {
  if (!Array.isArray(obj)) return false;
  return obj.every((item) => isBook(item));
};
