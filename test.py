from rust_queue import PriorityQueue
import unittest  # Hacker rank


class Test(unittest.TestCase):
    def test_iter(self):
        self.large_test_tree()
        # 20, 19, 18.5, 18, 18, 18, 17, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1

        # Test tree 1
        self.queue.push(5)
        self.queue.push(0)
        self.queue.push(10)
        self.queue.push(10)
        self.queue.push(5)
        self.queue.push(3)
        self.queue.push(6)
        self.queue.push(2)
        # 10, 10, 6, 5, 5, 3, 2, 0

        queue_iter = iter(self.queue)
        self.assertEqual(20, next(queue_iter))
        self.assertEqual(19, next(queue_iter))
        self.assertEqual(18.5, next(queue_iter))
        self.assertEqual(18, next(queue_iter))
        self.assertEqual(18, next(queue_iter))
        self.assertEqual(18, next(queue_iter))
        self.assertEqual(17, next(queue_iter))
        self.assertEqual(17, next(queue_iter))
        self.assertEqual(16, next(queue_iter))
        self.assertEqual(15, next(queue_iter))
        self.assertEqual(14, next(queue_iter))
        self.assertEqual(13, next(queue_iter))
        self.assertEqual(12, next(queue_iter))
        self.assertEqual(11, next(queue_iter))
        self.assertEqual(10, next(queue_iter))
        self.assertEqual(10, next(queue_iter))
        self.assertEqual(10, next(queue_iter))
        self.assertEqual(9, next(queue_iter))
        self.assertEqual(8, next(queue_iter))
        self.assertEqual(7, next(queue_iter))
        self.assertEqual(6, next(queue_iter))
        self.assertEqual(6, next(queue_iter))
        self.assertEqual(5, next(queue_iter))
        self.assertEqual(5, next(queue_iter))
        self.assertEqual(5, next(queue_iter))
        self.assertEqual(4, next(queue_iter))
        self.assertEqual(3, next(queue_iter))
        self.assertEqual(3, next(queue_iter))
        self.assertEqual(2, next(queue_iter))
        self.assertEqual(2, next(queue_iter))
        self.assertEqual(1, next(queue_iter))
        self.assertEqual(0, next(queue_iter))
        self.assertRaises(StopIteration, queue_iter.__next__)
        self.assertRaises(StopIteration, queue_iter.__next__)

    def test_indexing(self):
        self.small_test_tree()

        self.assertEqual(10, self.queue[0])
        self.assertEqual(6, self.queue[1])
        self.assertEqual(5, self.queue[2])
        self.assertEqual(5, self.queue[3])
        self.assertEqual(3, self.queue[4])
        self.assertEqual(2, self.queue[5])
        self.assertEqual(0, self.queue[6])
        self.assertRaises(IndexError, lambda: self.queue[7])

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

    def test_length_tracking(self):
        for _ in range(5):
            self.queue.push(None)
        self.assertEqual(5, len(self.queue))

        # Peek does not modify length
        self.queue.peek()
        self.assertEqual(5, len(self.queue))

        self.queue.pop()
        self.assertEqual(4, len(self.queue))
        # del self.queue[0]  # Unimpled
        # self.assertEqual(3, len(self.queue))
        self.queue.clear()
        self.assertEqual(0, len(self.queue))
        self.queue.pop()
        self.assertEqual(0, len(self.queue))

    def test_other(self):
        """For things that don't really get tested elsewhere"""
        self.medium_test_tree()

        self.assertEqual(10, self.queue.pop())
        self.assertEqual(10, self.queue.pop())
        self.assertEqual(10, self.queue.peek())
        self.assertEqual(10, self.queue.pop())

        self.assertEqual(6, self.queue.peek())
        self.assertEqual(6, self.queue.pop())
        self.assertEqual(6, self.queue.pop())

        # Uncomment when impled
        # for _ in range(3):
        #     del self.queue[0]
        # Comment when impled
        for _ in range(3):
            self.queue.pop()
        self.assertEqual(5, self.queue.pop())
        self.assertEqual(3, self.queue.pop())

        self.queue.clear()
        self.assertEqual(None, self.queue.pop())
        self.assertEqual(None, self.queue.peek())
        # del self.queue[0]?

    # def test_print(self):
    #     self.assertEqual("[]", str(self.queue))
    #     self.queue.push(5)
    #     self.queue.push(10)
    #     self.queue.push(5)
    #     self.queue.push(1)
    #     self.queue.push(3)
    #     self.assertEqual("[10, 5, 5, 3, 1]", str(self.queue))

    def small_test_tree(self):
        """10, 6, 5, 5, 3, 2, 0"""
        self.queue.push(5)
        self.queue.push(0)
        self.queue.push(3)
        self.queue.push(2)
        self.queue.push(10)
        self.queue.push(5)
        self.queue.push(6)

    def medium_test_tree(self):
        """10, 10, 10, 6, 6, 5, 5, 5, 5, 3, 3, 2, 2, 0, 0"""
        self.small_test_tree()
        # Test tree 1
        self.queue.push(5)
        self.queue.push(0)
        self.queue.push(10)
        self.queue.push(10)
        self.queue.push(5)
        self.queue.push(3)
        self.queue.push(6)
        self.queue.push(2)
        # 10, 10, 6, 5, 5, 3, 2, 0

    def large_test_tree(self):
        """20, 19, 18.5, 18, 18, 18, 17, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1"""
        # Test tree 2
        self.queue.push(10)
        self.queue.push(6)
        self.queue.push(14)
        self.queue.push(4)
        self.queue.push(8)
        self.queue.push(12)
        self.queue.push(16)
        self.queue.push(3)
        self.queue.push(5)
        self.queue.push(7)
        self.queue.push(9)
        self.queue.push(11)
        self.queue.push(13)
        self.queue.push(15)
        self.queue.push(17)
        ###################
        self.queue.push(1)
        self.queue.push(2)
        self.queue.push(18)
        self.queue.push(17)
        self.queue.push(19)
        self.queue.push(20)
        self.queue.push(18)
        self.queue.push(18)
        self.queue.push(18.5)

    def setUp(self) -> None:
        self.queue = PriorityQueue()


if __name__ == '__main__':
    unittest.main()
