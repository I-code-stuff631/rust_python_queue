from python_extension import PriorityQueue
import unittest  # Hacker rank


class Test(unittest.TestCase):
    def test_iter(self):
        # Test tree 1
        self.queue.push(5)
        self.queue.push(0)
        self.queue.push(10)
        self.queue.push(10)
        self.queue.push(5)
        self.queue.push(3)
        self.queue.push(6)
        self.queue.push(2)

        queue_iter = iter(self.queue)
        self.assertEqual(10, next(queue_iter))
        self.assertEqual(10, next(queue_iter))
        self.assertEqual(6, next(queue_iter))
        self.assertEqual(5, next(queue_iter))
        self.assertEqual(5, next(queue_iter))
        self.assertEqual(3, next(queue_iter))
        self.assertEqual(2, next(queue_iter))
        self.assertEqual(0, next(queue_iter))
        self.assertRaises(StopIteration, queue_iter.__next__)
        self.assertRaises(StopIteration, queue_iter.__next__)
        del queue_iter
    #
    #     self.queue = PriorityQueue()
    #     # Test tree 2
    #     self.queue.push()
    #     self.queue.push()

    def test_indexing(self):
        self.queue.push(7)
        self.queue.push(9)
        self.queue.push(4)
        self.queue.push(5)
        self.queue.push(4)
        self.assertEqual(9, self.queue[0])
        self.assertEqual(7, self.queue[1])
        self.assertEqual(5, self.queue[2])
        self.assertEqual(4, self.queue[3])
        self.assertEqual(4, self.queue[4])
        self.assertRaises(IndexError, lambda: self.queue[5])

    def test_contains(self):
        self.queue.push(1)
        self.queue.push(10)
        self.queue.push(5)
        self.assertTrue(5 in self.queue)
        self.assertTrue(10 in self.queue)
        self.assertTrue(1 in self.queue)
        self.queue.clear()
        self.assertFalse(5 in self.queue)
        self.assertFalse(10 in self.queue)
        self.assertFalse(1 in self.queue)

    # def test_print(self):
    #     self.assertEqual("[]", str(self.queue))
    #     self.queue.push(5)
    #     self.queue.push(10)
    #     self.queue.push(5)
    #     self.queue.push(1)
    #     self.queue.push(3)
    #     self.assertEqual("[10, 5, 5, 3, 1]", str(self.queue))

    def test_length_tracking(self):
        self.queue.push(5)
        self.queue.push(10)
        self.queue.push(5)
        self.queue.push(1)
        self.queue.push(3)
        self.assertEqual(5, len(self.queue))

        # Peek does not modify length
        self.queue.peek()
        self.assertEqual(5, len(self.queue))

        del self.queue[0]
        self.assertEqual(4, len(self.queue))
        # self.queue.pop()
        # self.queue.pop()
        # self.assertEqual(3, len(self.queue))
        # self.queue.clear()
        # self.assertEqual(0, len(self.queue))
        # self.queue.pop()
        # self.assertEqual(0, len(self.queue))

    def setUp(self) -> None:
        self.queue = PriorityQueue()


def test():
    queue = PriorityQueue()
    queue.push(1)
    queue.push(1)
    queue.peek()


if __name__ == '__main__':
    test()
    # unittest.main()
