/// ВАЖНО: все задания выполнять не обязательно. Что получится то получится сделать.

/// Задание 1
/// Почему фунция example1 зависает?
/// Для запуска задач используется рантайм с одним потоком, а значит задачи не могут выполняться одновременно
/// a1 запускает бесконечный цикл, который завершится только при получении сообщения из a2, но этого никогда не произойдет
/// a1 полностью занимает единственный поток, где вызывается метод try_recv(), который пытается немедленно получить сообщение из канала и возвращает ошибку, если он пуст, не приостанавливая задачу
/// a2, которая должна отправить сообщение, должна выполняться на том же потоке
/// Но поскольку a2 никогда не получит доступ к потоку для выполнения, то сообщение никогда не отправится в канал
/// А a1 никогда не получит сообщение из пустого канала, и цикл никогда не завершится
/// Главный поток, ожидающий завершение h1 и h2, зависает навсегда
/// Для решения проблемы можно заменить метод try_recv() на recv().await, который приостановит a1, освободив поток, до тех пор, пока в канале не появится сообщение.
fn example1() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .build()
        .unwrap();
    let (sd, mut rc) = tokio::sync::mpsc::unbounded_channel();

    let a1 = async move {
        loop {
            if let Ok(p) = rc.try_recv() {
                println!("{}", p);
                break;
            }
        }
    };
    let h1 = rt.spawn(a1);

    let a2 = async move {
        let _ = sd.send("message");
    };
    let h2 = rt.spawn(a2);
    while !(h1.is_finished() || h2.is_finished()) {}

    println!("execution completed");
}

#[derive(Clone)]
struct Example2Struct {
    value: u64,
    ptr: *const u64,
}

/// Задание 2
/// Какое число тут будет распечатано 32 64 или 128 и почему?
/// Будет распечатано 64
/// Это происходит из-за использования указателей *const для чтения освобожденной памяти, что является неопределенным поведением
/// Изначально при создании t1 поле value равно 64, ptr указывает на переменную num со значением 32
/// После строки 't1.ptr = &t1.value' t1.ptr указывает на адрес, где хранится собственное значение t1.value
/// При использовании метода clone() у t1 все значения полностью копируются в t2, то есть t2.value = 64, а t2.ptr указывает на тоже место памяти, куда указывал и t1.ptr, то есть на t1.value
/// drop(t1) освобождает место, отведенное под t1, но ничего не перезаписало это место
/// Поэтому при попытке прочитать значение по адресу, на который указывает t2.ptr, выводится то значение, которое было в этой ячейке памяти, то есть 64
/// t2.value = 128 никак на это не влияет, так как t2.ptr указывает не на него 
fn example2() {

    let num = 32;

    let mut t1 = Example2Struct {
        value: 64,
        ptr: &num,
    };

    t1.ptr = &t1.value;

    let mut t2 = t1.clone();

    drop(t1);

    t2.value = 128;

    unsafe {
        println!("{}", t2.ptr.read());
    }

    println!("execution completed");
}

/// Задание 3
/// Почему время исполнения всех пяти заполнений векторов разное (под linux)?
fn example3() {
    let capacity = 10000000u64;

    let start_time = std::time::Instant::now();
    let mut my_vec1 = Vec::new();
    for i in 0u64..capacity {
        my_vec1.insert(i as usize, i);
    }
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    let start_time = std::time::Instant::now();
    let mut my_vec2 = Vec::with_capacity(capacity as usize);
    for i in 0u64..capacity {
        my_vec2.insert(i as usize, i);
    }
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    let start_time = std::time::Instant::now();
    let mut my_vec3 = vec![6u64; capacity as usize];
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    let start_time = std::time::Instant::now();
    for mut elem in my_vec3 {
        elem = 7u64;
    }
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    let start_time = std::time::Instant::now();
    let my_vec4 = vec![0u64; capacity as usize];
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    println!("execution completed");
}

/// Задание 4
/// Почему такая разница во времени выполнения example4_async_mutex и example4_std_mutex?
/// Разница во времени выполнения связана с работой Mutex из стандартной библиотеки и Tokio.
/// При попытке получить доступ к std::sync::Mutex, который заблокирован другим потоком, текущий поток заблокируется и будет ждать, пока не сможет получить доступ к Mutex.
/// При использовании tokio::sync::Mutex при попытке заблокировать уже захваченный Mutex, поток не заблокируется.
/// Вместо этого задача будет приостановлена и поставлена в очередь ожидания Mutex, а поток при этом становится свободен для выполнения других задач.
/// Но с tokio::sync::Mutex появляются дополнительные накладные расходы, связанные с управлением асинхронными задачами планировщиком:
/// приостановка, возобновление, переключение между задачами, работа с очередями ожидания.
/// Всё это сильно увеличивает время выполнения при очень частых и быстрых захватах
/// и освобождениях Mutex под высокой конкуренцией, как в данном примере.
async fn example4_async_mutex(tokio_protected_value: std::sync::Arc<tokio::sync::Mutex<u64>>) {
    for _ in 0..1000000 {
        let mut value = *tokio_protected_value.clone().lock().await;
        value = 4;
    }
}

async fn example4_std_mutex(protected_value: std::sync::Arc<std::sync::Mutex<u64>>) {
    for _ in 0..1000000 {
        let mut value = *protected_value.clone().lock().unwrap();
        value = 4;
    }
}

fn example4() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .build()
        .unwrap();

    let mut tokio_protected_value = std::sync::Arc::new(tokio::sync::Mutex::new(0u64));

    let start_time = std::time::Instant::now();
    let h1 = rt.spawn(example4_async_mutex(tokio_protected_value.clone()));
    let h2 = rt.spawn(example4_async_mutex(tokio_protected_value.clone()));
    let h3 = rt.spawn(example4_async_mutex(tokio_protected_value.clone()));

    while !(h1.is_finished() || h2.is_finished() || h3.is_finished()) {}
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    let protected_value = std::sync::Arc::new(std::sync::Mutex::new(0u64));

    let start_time = std::time::Instant::now();
    let h1 = rt.spawn(example4_std_mutex(protected_value.clone()));
    let h2 = rt.spawn(example4_std_mutex(protected_value.clone()));
    let h3 = rt.spawn(example4_std_mutex(protected_value.clone()));

    while !(h1.is_finished() || h2.is_finished() || h3.is_finished()) {}
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    println!("execution completed");
}

/// Задание 5
/// В чем ошибка дизайна? Каких тестов не хватает? Есть ли лишние тесты?
/// Ошибка связана с возможностью изменить поля a, b и c в любой момент, когда area и perimeter вычисляются при первом вызове соответствующих методов
/// Если изменить какую-то точку после вызова area() или perimeter(), то их последующие вызовы будут возвращать неактуальные значения
/// Лучше сделать эти поля приватными, а для их изменения предоставить методы set_#(), которые будут устанавливать значение None в поля area и perimeter
/// Тогда после изменения координат площадь и периметр будут пересчитаны при следующем вызове area() и perimeter()
/// Не хватает тестов, которые проверяли бы значения площади и периметра после изменения точек, это могло бы выявить эту ошибку
/// Также можно добавить тесты для разных видов треугольников и координат с отрицательными значениями, чтобы быть уверенным в правильности реализации этих методов,
/// хотя сами формулы должны корректно работать с любыми значениями координат
/// Лишних тестов нет, но строка println!("{}",t.area()); не выполняет никаких проверок и её лучше заменить на assert
/// 
/// При добавлении тестов было замечено отсутствие необходимых скобок в расчёте площади, без них только благодаря удаче площадь получалась правильной 
mod example5 {
    pub struct Triangle {
        a: (f32, f32),
        b: (f32, f32),
        c: (f32, f32),
        area: Option<f32>,
        perimeter: Option<f32>,
    }

    impl Triangle {
        // Изменение координат точек с установкой None в поля area и perimeter
        pub fn set_a(&mut self, a: (f32, f32)) {
            self.a = a;
            self.area = None;
            self.perimeter = None;
        }
        pub fn set_b(&mut self, b: (f32, f32)) {
            self.b = b;
            self.area = None;
            self.perimeter = None;
        }
        pub fn set_c(&mut self, c: (f32, f32)) {
            self.c = c;
            self.area = None;
            self.perimeter = None;
        }
        
        //calculate area which is a positive number
        pub fn area(&mut self) -> f32 {
            if let Some(area) = self.area {
                area
            } else {
                // Добавлены скобки, группирующие разность
                self.area = Some(f32::abs(
                    (1f32 / 2f32) * ((self.a.0 - self.c.0) * (self.b.1 - self.c.1)
                        - (self.b.0 - self.c.0) * (self.a.1 - self.c.1)),
                ));
                self.area.unwrap()
            }
        }

        fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
            f32::sqrt((b.0 - a.0) * (b.0 - a.0) + (b.1 - a.1) * (b.1 - a.1))
        }

        //calculate perimeter which is a positive number
        pub fn perimeter(&mut self) -> f32 {
            if let Some(perimeter) = self.perimeter {
                return perimeter;
            } else {
                self.perimeter = Some(
                    Triangle::dist(self.a, self.b)
                        + Triangle::dist(self.b, self.c)
                        + Triangle::dist(self.c, self.a),
                );
                self.perimeter.unwrap()
            }
        }

        //new makes no guarantee for a specific values of a,b,c,area,perimeter at initialization
        pub fn new() -> Triangle {
            Triangle {
                a: (0f32, 0f32),
                b: (0f32, 0f32),
                c: (0f32, 0f32),
                area: None,
                perimeter: None,
            }
        }
    }
}

#[cfg(test)]
mod example5_tests {
    use super::example5::Triangle;

    // Заменим изменение координат через прямое присваивание на использование t.set_#()
    #[test]
    fn test_area() {
        let mut t = Triangle::new();

        t.set_a((0f32, 0f32));
        t.set_b((0f32, 0f32));
        t.set_c((0f32, 0f32));

        assert!(t.area() == 0f32);

        let mut t = Triangle::new();

        t.set_a((0f32, 0f32));
        t.set_b((0f32, 1f32));
        t.set_c((1f32, 0f32));

        assert!(t.area() == 0.5);

        let mut t = Triangle::new();

        t.set_a((0f32, 0f32));
        t.set_b((0f32, 1000f32));
        t.set_c((1000f32, 0f32));
        // Замена println на assert 
        assert!(t.area() == 500000f32);

        // Тест на перерасчёт площади после изменения координат
        t.set_a((0f32, 0f32));
        t.set_b((0f32, 1f32));
        t.set_c((1f32, 0f32));

        assert!(t.area() == 0.5);

        // Тест для остроугольного треугольника
        t.set_a((0f32, 0f32));
        t.set_b((4f32, 0f32));
        t.set_c((2f32, 3f32));
        assert!(t.area() == 6.0f32);

        // Tест для тупоугольного треугольника
        t.set_a((0f32, 0f32));
        t.set_b((5f32, 0f32));
        t.set_c((6f32, 1f32));
        assert!(t.area() == 2.5);

        // Тест для точек, расположенных в разных четвертях координатной плоскости
        t.set_a((2f32, 3f32));
        t.set_b((-1f32, 4f32));
        t.set_c((-3f32, -2f32));
        assert!(t.area() == 10f32);

        // Тест для точек, расположенных на одной прямой
        t.set_a((0f32, 0f32));
        t.set_b((2f32, 0f32));
        t.set_c((5f32, 0f32));
        assert!(t.area() == 0f32);
    }

    // Такая же замена на t.set_#()
    #[test]
    fn test_perimeter() {
        let mut t = Triangle::new();

        t.set_a((0f32, 0f32));
        t.set_b((0f32, 0f32));
        t.set_c((0f32, 0f32));

        assert!(t.perimeter() == 0f32);

        let mut t = Triangle::new();

        t.set_a((0f32, 0f32));
        t.set_b((0f32, 1f32));
        t.set_c((1f32, 0f32));

        assert!(t.perimeter() == 2f32 + f32::sqrt(2f32));

        // Тест на перерасчёт периметра после изменения координат
        t.set_a((0f32, 0f32));
        t.set_b((0f32, 3f32));
        t.set_c((4f32, 0f32));

        assert!(t.perimeter() == 12f32);
        
        // Тест для остроугольного треугольника
        t.set_a((0f32, 0f32));
        t.set_b((4f32, 0f32));
        t.set_c((2f32, 3f32));
        assert!(t.perimeter() == 4f32 + 2f32 * f32::sqrt(13f32));

        // Tест для тупоугольного треугольника
        t.set_a((0f32, 0f32));
        t.set_b((5f32, 0f32));
        t.set_c((6f32, 1f32));
        assert!(t.perimeter() == 5f32 + f32::sqrt(2f32) + f32::sqrt(37f32));

        // Тест для точек, расположенных в разных четвертях координатной плоскости
        t.set_a((2f32, 3f32));
        t.set_b((-1f32, 4f32));
        t.set_c((-3f32, -2f32));
        assert!(t.perimeter() == f32::sqrt(10f32) + f32::sqrt(40f32) + f32::sqrt(50f32));

        // Тест для точек, расположенных на одной прямой
        t.set_a((0f32, 0f32));
        t.set_b((2f32, 0f32));
        t.set_c((5f32, 0f32));
        assert!(t.perimeter() == 10f32);
    }
}

fn main() {
    example4();
}